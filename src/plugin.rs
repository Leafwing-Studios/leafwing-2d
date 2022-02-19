//! Tools for using two-dimensional coordinates within `bevy` games

use crate::orientation::{Direction, Rotation};
use crate::position::{Coordinate, Position};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::Quat;
use bevy_transform::components::{GlobalTransform, Transform};
use std::marker::PhantomData;

/// A [`Bundle`] of components that conveniently represents the state of entities living in 2-dimensional space
///
/// When used with other bundles (like a `SpriteBundle`), be aware that duplicate components (like [`Transform`])
/// will take the value of the last.
#[derive(Bundle, Clone, Debug, Default)]
pub struct TwoDimBundle<C: Coordinate> {
    /// The 2-dimensional [`Position`] of the entity
    ///
    /// This is automatically converted into a [`Transform`]'s translation
    pub position: Position<C>,
    /// Which way the entity is facing, stored as an angle from due north
    pub rotation: Rotation,
    /// Which way the entity is facing, stored as a unit vector
    pub direction: Direction,
    /// The local 3-dimensional position / rotation / scale of this entity
    pub transform: Transform,
    /// The absolute position / rotation / scale of this entity
    ///
    /// Can be modified by the entity's parent
    pub global_transform: GlobalTransform,
}

/// Ensures that two-dimensional [`Position`], [`Direction`] and [`Rotation`] components are synchronized with the [`Transform`] equivalent
///
/// The type paramter `C` is the coordinate type used in [`Position`]
///
/// System labels are stored in [`TwoDimSystem`], which describes the working of this plugin in more depth.
#[derive(Default, Debug)]
pub struct TwoDimPlugin<C: Coordinate> {
    _phantom: PhantomData<C>,
}

/// [`SystemLabel`] for [`TwoDimPlugin`]
///
/// These labels are executed in sequence.
#[derive(SystemLabel, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TwoDimSystem {
    /// Synchronizes the [`Direction`] and [`Rotation`] of all entities
    ///
    /// If [`Direction`] and [`Rotation`] are desynced, whichever one was changed will be used and the other will be made consistent.
    /// If both were changed, [`Rotation`] will be prioritized
    SyncDirectionRotation,
    /// Synchronizes the [`Rotation`] and [`Position`] of each entity with its [`Transform`]
    ///
    /// Not all components are needed for this system to do its work.
    SyncTransform,
}

impl<C: Coordinate> Plugin for TwoDimPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            sync_direction_and_rotation.label(TwoDimSystem::SyncDirectionRotation),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            sync_transform_with_2d::<C>
                .label(TwoDimSystem::SyncTransform)
                .after(TwoDimSystem::SyncDirectionRotation),
        );
    }
}

/// Synchronizes the [`Direction`] and [`Rotation`] of all entities
///
/// If [`Direction`] and [`Rotation`] are desynced, whichever one was changed will be used and the other will be made consistent.
/// If both were changed, [`Rotation`] will be prioritized
pub fn sync_direction_and_rotation(mut query: Query<(&mut Direction, &mut Rotation)>) {
    for (mut direction, mut rotation) in query.iter_mut() {
        if rotation.is_changed() {
            let new_direction: Direction = (*rotation).into();
            // These checks are required to avoid triggering change detection pointlessly,
            // which would create an infinite ping-pong effect
            if *direction != new_direction {
                *direction = new_direction;
            }
        } else if direction.is_changed() {
            // Direction can be [0, 0], so this conversion if fallible
            if let Ok(new_rotation) = (*direction).try_into() {
                if *rotation != new_rotation {
                    *rotation = new_rotation;
                }
            }
        }
    }
}

/// Synchronizes the [`Rotation`], [`Direction`] and [`Position`] of each entity with its [`Transform`] and vice versa
///
/// The [`Rotation`] and [`Direction`] will be synchronized, with whichever one has been changed taking priority.
/// If both have been changed, [`Rotation`] will take priority.
/// If both the [`Transform`] and its 2D analogue have been changed, the 2D version will take priority.
///
/// z-values of the [`Transform`] translation will not be modified.
/// Any off-axis rotation of the [`Transform`]'s rotation quaternion will be lost.
pub fn sync_transform_with_2d<C: Coordinate>(
    mut query: Query<
        (
            &mut Transform,
            Option<&mut Rotation>,
            Option<&mut Direction>,
            Option<&mut Position<C>>,
        ),
        Or<(With<Rotation>, With<Position<C>>)>,
    >,
) {
    for (mut transform, maybe_rotation, maybe_direction, maybe_position) in query.iter_mut() {
        // Synchronize Rotation with Transform
        if let Some(mut rotation) = maybe_rotation {
            if rotation.is_changed() {
                let new_quat: Quat = (*rotation).into();

                if transform.rotation != new_quat {
                    transform.rotation = new_quat;
                }
            } else if transform.is_changed() {
                if let Ok(new_rotation) = transform.rotation.try_into() {
                    if *rotation != new_rotation {
                        *rotation = new_rotation;
                    }
                }
            }
        }

        // Synchronize Direction with Transform
        if let Some(mut direction) = maybe_direction {
            if direction.is_changed() {
                if let Ok(new_quat) = (*direction).try_into() {
                    if transform.rotation != new_quat {
                        transform.rotation = new_quat;
                    }
                }
            } else if transform.is_changed() {
                if *direction != transform.rotation.into() {
                    *direction = transform.rotation.into();
                }
            }
        }

        // Synchronize Position with Transform
        if let Some(mut position) = maybe_position {
            if position.is_changed() {
                let new_x: f32 = position.x.clone().into();
                if transform.translation.x != new_x {
                    transform.translation.x = new_x;
                }

                let new_y: f32 = position.y.clone().into();
                if transform.translation.y != new_y {
                    transform.translation.y = new_y;
                }
            } else if transform.is_changed() {
                let new_x: C = transform.translation.x.into();
                if position.x != new_x {
                    position.x = new_x;
                }

                let new_y: C = transform.translation.y.into();
                if position.y != new_y {
                    position.y = new_y;
                }
            }
        }
    }
}
