use crate::{
    system::{System, ThreadLocalExecution},
    Resources, SystemId, World,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet}, sync::{Mutex, Arc},
};

#[derive(Default)]
pub struct Schedule {
    pub(crate) stages: HashMap<Cow<'static, str>, Vec<Arc<Mutex<Box<dyn System>>>>>,
    pub(crate) stage_order: Vec<Cow<'static, str>>,
    pub(crate) system_ids: HashSet<SystemId>,
    generation: usize,
    last_initialize_generation: usize,
}

impl Schedule {
    pub fn add_stage(&mut self, stage: impl Into<Cow<'static, str>>) {
        let stage: Cow<str> = stage.into();
        if let Some(_) = self.stages.get(&stage) {
            panic!("Stage already exists: {}", stage);
        } else {
            self.stages.insert(stage.clone(), Vec::new());
            self.stage_order.push(stage);
        }
    }

    pub fn add_stage_after(
        &mut self,
        target: impl Into<Cow<'static, str>>,
        stage: impl Into<Cow<'static, str>>,
    ) {
        let target: Cow<str> = target.into();
        let stage: Cow<str> = stage.into();
        if let Some(_) = self.stages.get(&stage) {
            panic!("Stage already exists: {}", stage);
        }

        let target_index = self
            .stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage)| **stage == target)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Target stage does not exist: {}", target));

        self.stages.insert(stage.clone(), Vec::new());
        self.stage_order.insert(target_index + 1, stage);
    }

    pub fn add_stage_before(
        &mut self,
        target: impl Into<Cow<'static, str>>,
        stage: impl Into<Cow<'static, str>>,
    ) {
        let target: Cow<str> = target.into();
        let stage: Cow<str> = stage.into();
        if let Some(_) = self.stages.get(&stage) {
            panic!("Stage already exists: {}", stage);
        }

        let target_index = self
            .stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage)| **stage == target)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Target stage does not exist: {}", target));

        self.stages.insert(stage.clone(), Vec::new());
        self.stage_order.insert(target_index, stage);
    }

    pub fn add_system_to_stage(
        &mut self,
        stage_name: impl Into<Cow<'static, str>>,
        system: Box<dyn System>,
    ) -> &mut Self {
        let stage_name = stage_name.into();
        let systems = self
            .stages
            .get_mut(&stage_name)
            .unwrap_or_else(|| panic!("Stage does not exist: {}", stage_name));
        if self.system_ids.contains(&system.id()) {
            panic!(
                "System with id {:?} ({}) already exists",
                system.id(),
                system.name()
            );
        }
        self.system_ids.insert(system.id());
        systems.push(Arc::new(Mutex::new(system)));

        self.generation += 1;
        self
    }

    pub fn run(&mut self, world: &mut World, resources: &mut Resources) {
        for stage_name in self.stage_order.iter() {
            if let Some(stage_systems) = self.stages.get_mut(stage_name) {
                for system in stage_systems.iter_mut() {
                    #[cfg(feature = "profiler")]
                    crate::profiler::profiler_start(resources, system.name().clone());
                    let mut system = system.lock().unwrap();
                    system.update_archetype_access(world);
                    match system.thread_local_execution() {
                        ThreadLocalExecution::NextFlush => system.run(world, resources),
                        ThreadLocalExecution::Immediate => {
                            system.run(world, resources);
                            // NOTE: when this is made parallel a full sync is required here
                            system.run_thread_local(world, resources);
                        }
                    }
                    #[cfg(feature = "profiler")]
                    crate::profiler::profiler_stop(resources, system.name().clone());
                }

                // "flush"
                // NOTE: when this is made parallel a full sync is required here
                for system in stage_systems.iter_mut() {
                    let mut system = system.lock().unwrap();
                    match system.thread_local_execution() {
                        ThreadLocalExecution::NextFlush => {
                            system.run_thread_local(world, resources)
                        }
                        ThreadLocalExecution::Immediate => { /* already ran immediate */ }
                    }
                }
            }
        }
    }

    // TODO: move this code to ParallelExecutor
    pub fn initialize(&mut self, resources: &mut Resources) {
        if self.last_initialize_generation == self.generation {
            return;
        }

        for stage in self.stages.values_mut() {
            for system in stage.iter_mut() {
                let mut system = system.lock().unwrap();
                system.initialize(resources);
            }
        }

        self.last_initialize_generation = self.generation;
    }

    pub fn generation(&self) -> usize {
        self.generation
    }
}