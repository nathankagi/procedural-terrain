use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use super::model::{Material, Model};
use super::transform::{Transform, TransformRaw};

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

#[derive(Clone, Debug)]
pub struct Object {
    pub model: Rc<Model>,
    pub pipeline: Rc<wgpu::RenderPipeline>,
    pub material: Option<Rc<Material>>,
    pub transform: Transform,
}

#[derive(Debug)]
pub struct Batch {
    pub model: Rc<Model>,
    pub pipeline: Rc<wgpu::RenderPipeline>,
    pub material: Option<Rc<Material>>,
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
    let mut groups: HashMap<(usize, usize, Option<usize>), Vec<EntityId>> = HashMap::new();
    for (&id, object) in objects {
        let key = (
            Rc::as_ptr(&object.model) as usize,
            Rc::as_ptr(&object.pipeline) as usize,
            object.material.as_ref().map(|m| Rc::as_ptr(m) as usize),
        );
        groups.entry(key).or_default().push(id);
    }

    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by_key(|(key, _)| (key.1, key.2, key.0));

    let mut instances = Vec::new();
    let mut batches = Vec::with_capacity(groups.len());

    for (_, entities) in groups {
        let representative = &objects[&entities[0]];
        let model = representative.model.clone();
        let pipeline = representative.pipeline.clone();
        let material = representative.material.clone();

        let start = instances.len() as u32;
        for id in &entities {
            instances.push(objects[id].transform.to_raw());
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
