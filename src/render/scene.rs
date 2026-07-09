use std::collections::HashMap;
use std::ops::Range;

use super::transform::{Transform, TransformRaw};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PipelineHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub usize);

pub struct Registry<H> {
    by_name: HashMap<String, H>,
}

impl<H> Default for Registry<H> {
    fn default() -> Self {
        Self {
            by_name: HashMap::new(),
        }
    }
}

impl<H: Copy> Registry<H> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: impl Into<String>, handle: H) {
        self.by_name.insert(name.into(), handle);
    }

    pub fn get(&self, name: &str) -> Option<H> {
        self.by_name.get(name).copied()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

#[derive(Default)]
pub struct EntityAllocator {
    next: u64,
}

impl EntityAllocator {
    pub fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next);
        self.next += 1;
        id
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Object {
    pub model: ModelHandle,
    pub pipeline: PipelineHandle,
    pub material: Option<MaterialHandle>,
    pub transform: Transform,
}

#[derive(Debug)]
pub struct Batch {
    pub model: ModelHandle,
    pub pipeline: PipelineHandle,
    pub material: Option<MaterialHandle>,
    pub instance_range: Range<u32>,
}

#[derive(Default)]
pub struct BatchedScene {
    pub batches: Vec<Batch>,
    pub instances: Vec<TransformRaw>,
}

pub struct Scene {
    entities: EntityAllocator,
    objects: HashMap<EntityId, Object>,
    batched: BatchedScene,
    dirty: bool,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            entities: EntityAllocator::default(),
            objects: HashMap::new(),
            batched: BatchedScene::default(),
            dirty: true,
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, object: Object) -> EntityId {
        let id = self.entities.spawn();
        self.objects.insert(id, object);
        self.dirty = true;
        id
    }

    pub fn despawn(&mut self, id: EntityId) {
        self.objects.remove(&id);
        self.dirty = true;
    }

    pub fn get(&self, id: EntityId) -> Option<&Object> {
        self.objects.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Object> {
        self.dirty = true;
        self.objects.get_mut(&id)
    }

    pub fn batches(&mut self) -> (&BatchedScene, bool) {
        let recomputed = self.dirty;
        if self.dirty {
            self.batched = build_batches(&self.objects);
            self.dirty = false;
        }
        (&self.batched, recomputed)
    }
}

fn build_batches(objects: &HashMap<EntityId, Object>) -> BatchedScene {
    let mut groups: HashMap<(ModelHandle, PipelineHandle, Option<MaterialHandle>), Vec<EntityId>> =
        HashMap::new();
    for (&id, object) in objects {
        groups
            .entry((object.model, object.pipeline, object.material))
            .or_default()
            .push(id);
    }

    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by_key(|(key, _)| (key.1.0, key.2.map(|m| m.0), key.0.0));

    let mut instances = Vec::new();
    let mut batches = Vec::with_capacity(groups.len());

    for ((model, pipeline, material), entities) in groups {
        let start = instances.len() as u32;
        for id in entities {
            instances.push(objects[&id].transform.to_raw());
        }
        let end = instances.len() as u32;
        batches.push(Batch {
            model,
            pipeline,
            material,
            instance_range: start..end,
        });
    }

    BatchedScene { batches, instances }
}
