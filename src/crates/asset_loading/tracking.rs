//! A high-level way to load collections of asset handles as resources.

use super::*;
use std::{collections::VecDeque, path::Path};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ResourceHandles>();
    app.add_systems(PreUpdate, load_resource_assets);
}

pub trait LoadResource {
    /// This will load the [`Resource`] as an [`Asset`]. When all of its asset dependencies
    /// have been loaded, it will be inserted as a resource. This ensures that the resource only
    /// exists when the assets are ready.
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self;

    /// This will load the [`Resource`] as an [`Asset`] from the specified path. When all of
    /// its asset dependencies have been loaded, it will be inserted as a resource. This
    /// ensures that the resource only exists when the assets are ready.
    fn load_resource_from_path<T: Resource + Asset + Clone>(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &mut Self;
}

impl LoadResource for App {
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self {
        self.init_asset::<T>();
        let world = self.world_mut();
        let value = T::from_world(world);
        let assets = world.resource::<AssetServer>();
        let handle = assets.add(value);
        let mut handles = world.resource_mut::<ResourceHandles>();
        handles.push_handle(handle);
        self
    }

    fn load_resource_from_path<T: Resource + Asset + Clone>(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &mut Self {
        self.init_asset::<T>();
        let handle = {
            let world = self.world_mut();
            let assets = world.resource::<AssetServer>();
            let handle: Handle<T> = assets.load::<T>(path.as_ref());
            let src_handle = handle.clone();
            let mut handles = world.resource_mut::<ResourceHandles>();
            handles.push_handle(handle);
            src_handle
        };

        // If we are running in dev mode on desktop where hot-reloading of assets is possible
        // then watch the asset and update the resource if it changes.
        #[cfg(feature = "dev_native")]
        self.add_systems(
            Update,
            move |mut commands: Commands,
                  mut events: EventReader<AssetEvent<T>>,
                  assets: Res<Assets<T>>| {
                for event in events.read() {
                    match event {
                        AssetEvent::Modified { id } if *id == handle.id() => {
                            if let Some(asset) = assets.get(&handle) {
                                commands.insert_resource(asset.clone());
                            }
                        }
                        _ => {}
                    }
                }
            },
        );

        self
    }
}

/// A function that inserts a loaded resource.
type InsertLoadedResource = fn(&mut World, &UntypedHandle);

#[derive(Resource, Default)]
pub struct ResourceHandles {
    // Use a queue for waiting assets so they can be cycled through and moved to
    // `finished` one at a time.
    waiting: VecDeque<(UntypedHandle, InsertLoadedResource)>,
    finished: Vec<UntypedHandle>,
}

impl ResourceHandles {
    /// Returns true if all requested [`Asset`]s have finished loading and are available as [`Resource`]s.
    pub fn is_all_done(&self) -> bool {
        self.waiting.is_empty()
    }

    /// Adds an asset handle to the list of pending assets to be tracked and converted to resources
    /// on load.
    pub fn push_handle<T: Asset + Resource + Clone>(&mut self, handle: Handle<T>) {
        self.waiting.push_back((handle.untyped(), |world, handle| {
            let assets = world.resource::<Assets<T>>();
            if let Some(value) = assets.get(handle.id().typed::<T>()) {
                world.insert_resource(value.clone());
            }
        }));
    }
}

fn load_resource_assets(world: &mut World) {
    world.resource_scope(|world, mut resource_handles: Mut<ResourceHandles>| {
        world.resource_scope(|world, assets: Mut<AssetServer>| {
            for _ in 0..resource_handles.waiting.len() {
                let (handle, insert_fn) = resource_handles.waiting.pop_front().unwrap();
                if assets.is_loaded_with_dependencies(&handle) {
                    insert_fn(world, &handle);
                    resource_handles.finished.push(handle);
                } else {
                    resource_handles.waiting.push_back((handle, insert_fn));
                }
            }
        });
    });
}
