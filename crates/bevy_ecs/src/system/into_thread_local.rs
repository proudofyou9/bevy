pub use super::Query;
use crate::{
    resource::Resources,
    system::{System, SystemId, ThreadLocalExecution},
    TypeAccess,
};
use bevy_hecs::{ArchetypeComponent, World};
use std::{any::TypeId, borrow::Cow};

#[derive(Debug)]
pub(crate) struct ThreadLocalSystemFn<Func>
where
    Func: FnMut(&mut World, &mut Resources) + Send + Sync,
{
    pub func: Func,
    pub resource_access: TypeAccess<TypeId>,
    pub archetype_component_access: TypeAccess<ArchetypeComponent>,
    pub name: Cow<'static, str>,
    pub id: SystemId,
}

impl<Func> System for ThreadLocalSystemFn<Func>
where
    Func: FnMut(&mut World, &mut Resources) + Send + Sync,
{
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }

    fn update(&mut self, _world: &World) {}

    fn archetype_component_access(&self) -> &TypeAccess<ArchetypeComponent> {
        &self.archetype_component_access
    }

    fn resource_access(&self) -> &TypeAccess<TypeId> {
        &self.resource_access
    }

    fn thread_local_execution(&self) -> ThreadLocalExecution {
        ThreadLocalExecution::Immediate
    }

    fn run(&mut self, _world: &World, _resources: &Resources) {}

    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources) {
        (self.func)(world, resources);
    }

    fn initialize(&mut self, _world: &mut World, _resources: &mut Resources) {}

    fn id(&self) -> SystemId {
        self.id
    }

    fn is_initialized(&self) -> bool {
        true
    }
}

/// Converts `Self` into a thread local system
pub trait IntoThreadLocalSystem {
    fn thread_local_system(self) -> Box<dyn System>;
}

impl<F> IntoThreadLocalSystem for F
where
    F: FnMut(&mut World, &mut Resources) + Send + Sync + 'static,
{
    fn thread_local_system(mut self) -> Box<dyn System> {
        Box::new(ThreadLocalSystemFn {
            func: move |world, resources| (self)(world, resources),
            name: core::any::type_name::<F>().into(),
            id: SystemId::new(),
            resource_access: TypeAccess::default(),
            archetype_component_access: TypeAccess::default(),
        })
    }
}
