use bevy::{ecs::system::SystemId, prelude::*};
use foldhash::HashMap;

/// Generate a handler map.
macro_rules! handler_map {
    ($name:ident, $($funcs:ident),*) => {
        #[derive(Debug, Resource)]
        pub struct $name(pub(super) HashMap<&'static str, SystemId>);

        impl $name {
            pub fn insert(mut commands: Commands) {

                let handlers = [
                    $(
                        (stringify!($funcs), commands.register_one_shot_system($funcs))
                    ),*
                ];

                let mut map = HashMap::default();
                map.extend(handlers);

                commands.insert_resource($name(map));
            }
        }
    };
}

// TODO: validate that all assets use existing handler names
// at startup in debug mode.

handler_map! {ResponseHandlers, test, test2}

fn test(time: Res<Time>) {
    println!("Hello, world! {}", time.delta_seconds());
}

fn test2(time: Res<Time>) {
    println!("Hello, world2! {}", time.delta_seconds());
}
