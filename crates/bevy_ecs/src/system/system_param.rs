pub use bevy_hecs::SystemParam;

use crate::{
    ChangedRes, Commands, FromResources, Local, Query, QuerySet, QueryTuple, Res, ResMut, Resource,
    ResourceIndex, Resources, SystemState,
};
use bevy_hecs::{ArchetypeComponent, Fetch, Or, Query as HecsQuery, TypeAccess, World};
use parking_lot::Mutex;
use std::{any::TypeId, sync::Arc};

pub trait SystemParam: Sized {
    fn init(system_state: &mut SystemState, world: &World, resources: &mut Resources);
    /// # Safety
    /// This call might access any of the input parameters in an unsafe way. Make sure the data access is safe in
    /// the context of the system scheduler
    unsafe fn get_param(
        system_state: &mut SystemState,
        world: &World,
        resources: &Resources,
    ) -> Option<Self>;
}

impl<'a, Q: HecsQuery> SystemParam for Query<'a, Q> {
    #[inline]
    unsafe fn get_param(
        system_state: &mut SystemState,
        world: &World,
        _resources: &Resources,
    ) -> Option<Self> {
        let query_index = system_state.current_query_index;
        let world: &'a World = std::mem::transmute(world);
        let archetype_component_access: &'a TypeAccess<ArchetypeComponent> =
            std::mem::transmute(&system_state.query_archetype_component_accesses[query_index]);
        system_state.current_query_index += 1;
        Some(Query::new(world, archetype_component_access))
    }

    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state
            .query_archetype_component_accesses
            .push(TypeAccess::default());
        system_state
            .query_accesses
            .push(vec![<Q::Fetch as Fetch>::access()]);
        system_state
            .query_type_names
            .push(std::any::type_name::<Q>());
    }
}

impl<T: QueryTuple> SystemParam for QuerySet<T> {
    #[inline]
    unsafe fn get_param(
        system_state: &mut SystemState,
        world: &World,
        _resources: &Resources,
    ) -> Option<Self> {
        let query_index = system_state.current_query_index;
        system_state.current_query_index += 1;
        Some(QuerySet::new(
            world,
            &system_state.query_archetype_component_accesses[query_index],
        ))
    }

    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state
            .query_archetype_component_accesses
            .push(TypeAccess::default());
        system_state.query_accesses.push(T::get_accesses());
        system_state
            .query_type_names
            .push(std::any::type_name::<T>());
    }
}

impl<'a> SystemParam for &'a mut Commands {
    fn init(system_state: &mut SystemState, world: &World, _resources: &mut Resources) {
        system_state
            .commands
            .set_entity_reserver(world.get_entity_reserver())
    }

    #[inline]
    unsafe fn get_param(
        system_state: &mut SystemState,
        _world: &World,
        _resources: &Resources,
    ) -> Option<Self> {
        let commands: &'a mut Commands = std::mem::transmute(&mut system_state.commands);
        Some(commands)
    }
}

impl SystemParam for Arc<Mutex<Commands>> {
    fn init(system_state: &mut SystemState, world: &World, _resources: &mut Resources) {
        system_state.arc_commands.get_or_insert_with(|| {
            let mut commands = Commands::default();
            commands.set_entity_reserver(world.get_entity_reserver());
            Arc::new(Mutex::new(commands))
        });
    }

    #[inline]
    unsafe fn get_param(
        system_state: &mut SystemState,
        _world: &World,
        _resources: &Resources,
    ) -> Option<Self> {
        Some(system_state.arc_commands.as_ref().unwrap().clone())
    }
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state.resource_access.add_read(TypeId::of::<T>());
    }

    #[inline]
    unsafe fn get_param(
        _system_state: &mut SystemState,
        _world: &World,
        resources: &Resources,
    ) -> Option<Self> {
        Some(Res::new(
            resources.get_unsafe_ref::<T>(ResourceIndex::Global),
        ))
    }
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state.resource_access.add_write(TypeId::of::<T>());
    }

    #[inline]
    unsafe fn get_param(
        _system_state: &mut SystemState,
        _world: &World,
        resources: &Resources,
    ) -> Option<Self> {
        let (value, type_state) =
            resources.get_unsafe_ref_with_type_state::<T>(ResourceIndex::Global);
        Some(ResMut::new(value, type_state.mutated()))
    }
}

impl<'a, T: Resource> SystemParam for ChangedRes<'a, T> {
    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state.resource_access.add_read(TypeId::of::<T>());
    }

    #[inline]
    unsafe fn get_param(
        _system_state: &mut SystemState,
        _world: &World,
        resources: &Resources,
    ) -> Option<Self> {
        let (added, mutated) = resources.get_unsafe_added_and_mutated::<T>(ResourceIndex::Global);
        if *added.as_ptr() || *mutated.as_ptr() {
            Some(ChangedRes::new(
                resources.get_unsafe_ref::<T>(ResourceIndex::Global),
            ))
        } else {
            None
        }
    }
}

impl<'a, T: Resource + FromResources> SystemParam for Local<'a, T> {
    fn init(system_state: &mut SystemState, _world: &World, resources: &mut Resources) {
        system_state.resource_access.add_write(TypeId::of::<T>());
        if resources.get_local::<T>(system_state.id).is_none() {
            let value = T::from_resources(resources);
            resources.insert_local(system_state.id, value);
        }
    }

    #[inline]
    unsafe fn get_param(
        system_state: &mut SystemState,
        _world: &World,
        resources: &Resources,
    ) -> Option<Self> {
        Some(Local::new(resources, system_state.id))
    }
}

macro_rules! impl_system_param_tuple {
    ($($param: ident),*) => {
        #[allow(unused_variables)]
        impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            fn init(system_state: &mut SystemState, world: &World, resources: &mut Resources) {
                $($param::init(system_state, world, resources);)*
            }

            #[inline]
            unsafe fn get_param(
                system_state: &mut SystemState,
                world: &World,
                resources: &Resources,
            ) -> Option<Self> {
                Some(($($param::get_param(system_state, world, resources)?,)*))
            }
        }

        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(non_snake_case)]
        impl<$($param: SystemParam),*> SystemParam for Or<($(Option<$param>,)*)> {
            fn init(system_state: &mut SystemState, world: &World, resources: &mut Resources) {
                $($param::init(system_state, world, resources);)*
            }

            #[inline]
            unsafe fn get_param(
                system_state: &mut SystemState,
                world: &World,
                resources: &Resources,
            ) -> Option<Self> {
                let mut has_some = false;
                $(
                    let $param = $param::get_param(system_state, world, resources);
                    if $param.is_some() {
                        has_some = true;
                    }
                )*

                if has_some {
                    Some(Or(($($param,)*)))
                } else {
                    None
                }
            }
        }
    };
}

impl_system_param_tuple!();
impl_system_param_tuple!(A);
impl_system_param_tuple!(A, B);
impl_system_param_tuple!(A, B, C);
impl_system_param_tuple!(A, B, C, D);
impl_system_param_tuple!(A, B, C, D, E);
impl_system_param_tuple!(A, B, C, D, E, F);
impl_system_param_tuple!(A, B, C, D, E, F, G);
impl_system_param_tuple!(A, B, C, D, E, F, G, H);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
