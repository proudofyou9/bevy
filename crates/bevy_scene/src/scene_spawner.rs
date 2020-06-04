use crate::Scene;
use bevy_app::{EventReader, Events};
use bevy_asset::{AssetEvent, Assets, Handle};
use bevy_type_registry::TypeRegistry;
use legion::{
    entity::EntityIndex,
    guid_entity_allocator::GuidEntityAllocator,
    prelude::{Entity, Resources, World},
};
use std::{
    collections::{HashMap, HashSet},
    num::Wrapping,
};
use thiserror::Error;
use uuid::Uuid;

struct InstanceInfo {
    entity_map: HashMap<EntityIndex, EntityIndex>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct InstanceId(Uuid);

impl InstanceId {
    pub fn new() -> Self {
        InstanceId(Uuid::new_v4())
    }
}

#[derive(Default)]
pub struct SceneSpawner {
    loaded_scenes: HashSet<Handle<Scene>>,
    spawned_scenes: HashMap<Handle<Scene>, Vec<InstanceId>>,
    spawned_instances: HashMap<InstanceId, InstanceInfo>,
    scene_asset_event_reader: EventReader<AssetEvent<Scene>>,
    scenes_to_spawn: Vec<Handle<Scene>>,
    scenes_to_load: Vec<Handle<Scene>>,
}

#[derive(Error, Debug)]
pub enum SceneSpawnError {
    #[error("Scene contains an unregistered component.")]
    UnregisteredComponent { type_name: String },
    #[error("Scene does not exist. Perhaps it is still loading?")]
    NonExistentScene { handle: Handle<Scene> },
}

impl SceneSpawner {
    pub fn spawn(&mut self, scene_handle: Handle<Scene>) {
        self.scenes_to_spawn.push(scene_handle);
    }

    pub fn load(&mut self, scene_handle: Handle<Scene>) {
        self.scenes_to_load.push(scene_handle);
    }

    pub fn load_sync(
        &mut self,
        world: &mut World,
        resources: &Resources,
        scene_handle: Handle<Scene>,
    ) -> Result<(), SceneSpawnError> {
        Self::load_internal(world, resources, scene_handle, None)?;
        self.loaded_scenes.insert(scene_handle);
        Ok(())
    }

    pub fn spawn_sync(
        &mut self,
        world: &mut World,
        resources: &Resources,
        scene_handle: Handle<Scene>,
    ) -> Result<(), SceneSpawnError> {
        let instance_id = InstanceId::new();
        let mut instance_info = InstanceInfo {
            entity_map: HashMap::default(),
        };
        Self::load_internal(world, resources, scene_handle, Some(&mut instance_info))?;
        self.spawned_instances.insert(instance_id, instance_info);
        let spawned = self
            .spawned_scenes
            .entry(scene_handle)
            .or_insert_with(|| Vec::new());
        spawned.push(instance_id);
        Ok(())
    }

    fn load_internal(
        world: &mut World,
        resources: &Resources,
        scene_handle: Handle<Scene>,
        mut instance_info: Option<&mut InstanceInfo>,
    ) -> Result<(), SceneSpawnError> {
        let type_registry = resources.get::<TypeRegistry>().unwrap();
        let component_registry = type_registry.component.read().unwrap();
        let scenes = resources.get::<Assets<Scene>>().unwrap();
        let scene = scenes
            .get(&scene_handle)
            .ok_or_else(|| SceneSpawnError::NonExistentScene {
                handle: scene_handle,
            })?;

        for scene_entity in scene.entities.iter() {
            let entity_id = if let Some(ref mut instance_info) = instance_info {
                *instance_info
                    .entity_map
                    .entry(scene_entity.entity)
                    .or_insert_with(|| GuidEntityAllocator::new_entity_id())
            } else {
                scene_entity.entity
            };
            let mut entity = Entity::new(entity_id, Wrapping(1));
            // TODO: use EntityEntry when legion refactor is finished
            if world.get_entity_location(entity).is_none() {
                world
                    .entity_allocator
                    .push_next_ids((&[entity]).iter().cloned());
                entity = world.insert((), vec![()])[0];
            }
            for component in scene_entity.components.iter() {
                let component_registration = component_registry
                    .get_with_name(&component.type_name)
                    .ok_or_else(|| SceneSpawnError::UnregisteredComponent {
                        type_name: component.type_name.to_string(),
                    })?;
                component_registration.add_component_to_entity(world, resources, entity, component);
            }
        }
        Ok(())
    }

    pub fn update_spawned_scenes(
        &mut self,
        world: &mut World,
        resources: &Resources,
        scene_handles: &[Handle<Scene>],
    ) -> Result<(), SceneSpawnError> {
        for scene_handle in scene_handles {
            if let Some(spawned_instances) = self.spawned_scenes.get(scene_handle) {
                for instance_id in spawned_instances.iter() {
                    if let Some(instance_info) = self.spawned_instances.get_mut(instance_id) {
                        Self::load_internal(world, resources, *scene_handle, Some(instance_info))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_queued_scenes(
        &mut self,
        world: &mut World,
        resources: &Resources,
    ) -> Result<(), SceneSpawnError> {
        let scenes_to_load = self.scenes_to_load.drain(..).collect::<Vec<_>>();
        let mut non_existent_scenes = Vec::new();
        for scene_handle in scenes_to_load {
            match self.load_sync(world, resources, scene_handle) {
                Ok(_) => {}
                Err(SceneSpawnError::NonExistentScene { .. }) => {
                    non_existent_scenes.push(scene_handle)
                }
                Err(err) => return Err(err),
            }
        }

        self.scenes_to_load = non_existent_scenes;
        Ok(())
    }

    pub fn spawn_queued_scenes(
        &mut self,
        world: &mut World,
        resources: &Resources,
    ) -> Result<(), SceneSpawnError> {
        let scenes_to_spawn = self.scenes_to_spawn.drain(..).collect::<Vec<_>>();
        let mut non_existent_scenes = Vec::new();
        for scene_handle in scenes_to_spawn {
            match self.spawn_sync(world, resources, scene_handle) {
                Ok(_) => {}
                Err(SceneSpawnError::NonExistentScene { .. }) => {
                    non_existent_scenes.push(scene_handle)
                }
                Err(err) => return Err(err),
            }
        }

        self.scenes_to_spawn = non_existent_scenes;
        Ok(())
    }
}

pub fn scene_spawner_system(world: &mut World, resources: &mut Resources) {
    let mut scene_spawner = resources.get_mut::<SceneSpawner>().unwrap();
    let scene_asset_events = resources.get::<Events<AssetEvent<Scene>>>().unwrap();

    let mut updated_spawned_scenes = Vec::new();
    for event in scene_spawner
        .scene_asset_event_reader
        .iter(&scene_asset_events)
    {
        if let AssetEvent::Modified { handle } = event {
            if scene_spawner.loaded_scenes.contains(handle) {
                scene_spawner.load(*handle);
            }
            if scene_spawner.spawned_scenes.contains_key(handle) {
                updated_spawned_scenes.push(*handle);
            }
        }
    }

    scene_spawner.load_queued_scenes(world, resources).unwrap();
    scene_spawner.spawn_queued_scenes(world, resources).unwrap();
    scene_spawner
        .update_spawned_scenes(world, resources, &updated_spawned_scenes)
        .unwrap();
}
