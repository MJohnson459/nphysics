#![allow(missing_docs)] // For downcast.

use std::fmt::Debug;
use downcast::Any;

use na::{self, DVectorSlice, DVectorSliceMut, Real};

use math::{Force, Inertia, Isometry, Point, Velocity};
use object::{BodyPartHandle, BodyHandle};
use solver::IntegrationParameters;

/// The status of a body.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum BodyStatus {
    /// The body is disabled and ignored by the physics engine.
    Disabled,
    /// The body is static and thus cannot move.
    Static,
    /// The body is dynamic and thus can move and is subject to forces.
    Dynamic,
    /// The body is kinematic so its velocity is controlled by the user and it is not affected by forces and constraints.
    Kinematic,
}

/// The activation status of a body.
///
/// This controls whether a body is sleeping or not.
#[derive(Copy, Clone, Debug)]
pub struct ActivationStatus<N: Real> {
    threshold: Option<N>,
    energy: N,
}

impl<N: Real> ActivationStatus<N> {
    /// The default amount of energy bellow which a body can be put to sleep by nphysics.
    pub fn default_threshold() -> N {
        na::convert(0.01f64)
    }

    /// Create a new activation status initialised with the default activation threshold and is active.
    pub fn new_active() -> Self {
        ActivationStatus {
            threshold: Some(Self::default_threshold()),
            energy: Self::default_threshold() * na::convert(4.0),
        }
    }

    /// Create a new activation status initialised with the default activation threshold and is inactive.
    pub fn new_inactive() -> Self {
        ActivationStatus {
            threshold: Some(Self::default_threshold()),
            energy: N::zero(),
        }
    }

    /// Retuns `true` if the body is not asleep.
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.energy.is_zero()
    }

    /// The threshold bellow which the body can be put to sleep.
    ///
    /// A value of `None` indicates that the body cannot sleep.
    #[inline]
    pub fn deactivation_threshold(&self) -> Option<N> {
        self.threshold
    }

    /// Set the threshold bellow which the body can be put to sleep.
    ///
    /// A value of `None` prevents the body from sleeping.
    #[inline]
    pub fn set_deactivation_threshold(&mut self, threshold: Option<N>) {
        self.threshold = threshold
    }

    /// The current energy averaged through several frames.
    #[inline]
    pub fn energy(&self) -> N {
        self.energy
    }

    /// Sets the current average energy of the body.
    #[inline]
    pub fn set_energy(&mut self, energy: N) {
        self.energy = energy
    }
}

/// Trait implemented by all bodies supported by nphysics.
pub trait Body<N: Real>: Any + Send + Sync {
    /// Returns `true` if this body is the ground.
    fn is_ground(&self) -> bool {
        false
    }

    /// Updates the kinematics, e.g., positions and jacobians, of this body.
    fn update_kinematics(&mut self);

    /// Reset the timestep-specific dynamic information of this body.
    fn clear_dynamics(&mut self);

    /// Applies a generalized displacement to this body.
    fn apply_displacement(&mut self, disp: &[N]);

    /// The number of degrees of freedom (DOF) of this body, taking its status into account.
    ///
    /// In particular, this returns 0 for any body with a status different than `BodyStatus::Dynamic`.
    #[inline]
    fn status_dependent_ndofs(&self) -> usize {
        if self.is_dynamic() {
            self.ndofs()
        } else {
            0
        }
    }

    /// The velocity of the specified body part, taking this body status into account.
    ///
    /// This will return a zero velocity for any body with a status different than `BodyStatus::Dynamic`.
    fn status_dependent_body_part_velocity(&self, part: &BodyPart<N>) -> Velocity<N> {
        if self.is_dynamic() {
            part.velocity()
        } else {
            Velocity::zero()
        }
    }

    /// Sets the tag associated to this body and its body parts.
    ///
    /// This is should not be called explicitly by user code. This is called
    /// by the world when the body is added to the world.
    /// If `tag` is `None`, all handles of this body and body parts should be
    /// reset to `None` as well.
    fn set_handle(&mut self, handle: Option<BodyHandle>);

    /// The handle of this body.
    fn handle(&self) -> Option<BodyHandle>;

    /// The status of this body.
    fn status(&self) -> BodyStatus;

    /// Information regarding activation and deactivation (sleeping) of this body.
    fn activation_status(&self) -> &ActivationStatus<N>;

    /// Check if this body is active.
    fn is_active(&self) -> bool;

    /// Whether or not the status of this body is dynamic.
    fn is_dynamic(&self) -> bool;

    /// Whether or not the status of this body is kinematic.
    fn is_kinematic(&self) -> bool;

    /// Whether or not the status of this body is static.
    fn is_static(&self) -> bool;

    /// The number of degrees of freedom of this body.
    fn ndofs(&self) -> usize;

    /// The generalized accelerations at each degree of freedom of this body.
    fn generalized_acceleration(&self) -> DVectorSlice<N>;

    /// The generalized velocities of this body.
    fn generalized_velocity(&self) -> DVectorSlice<N>;

    /// The companion ID of this body.
    fn companion_id(&self) -> usize;

    /// Set the companion ID of this body (may be reinitialized by nphysics).
    fn set_companion_id(&mut self, id: usize);

    /// The mutable generalized velocities of this body.
    fn generalized_velocity_mut(&mut self) -> DVectorSliceMut<N>;

    /// Integrate the position of this body.
    fn integrate(&mut self, params: &IntegrationParameters<N>);

    // FIXME: should those directly be implemented only for bodies (to avoid duplicated code on
    // each body for activation)?
    /// Force the activation of this body.
    fn activate(&mut self);

    /// Force the activation of this body with the given level of energy.
    fn activate_with_energy(&mut self, energy: N);

    /// Put this body to sleep.
    fn deactivate(&mut self);

    /// A reference to the specified body part.
    fn part(&self, handle: BodyPartHandle) -> &BodyPart<N>;

    /// A mutable reference to the specified body part.
    fn part_mut(&mut self, handle: BodyPartHandle) -> &mut BodyPart<N>;

    /// Returns `true` if `handle` is a valid handle for one of this body's part.
    fn contains_part(&self, handle: BodyPartHandle) -> bool;

    /// Convert generalized forces applied to this body part into generalized accelerations.
    fn inv_mass_mul_generalized_forces(&self, out: &mut [N]);

    /// Convert a force applied to the center of mass of this body part into generalized force.
    fn body_part_jacobian_mul_force(&self, part: &BodyPart<N>, force: &Force<N>, out: &mut [N]);

    /// Convert a force applied to this body part's center of mass into generalized accelerations.
    fn inv_mass_mul_body_part_force(&self, part: &BodyPart<N>, force: &Force<N>, out: &mut [N]);
}

/// Trait implemented by each part of a body supported by nphysics.
pub trait BodyPart<N: Real>: Any + Send + Sync {
    /// Returns `true` if this body part is the ground.
    fn is_ground(&self) -> bool;

    /// The handle of this body part.
    fn handle(&self) -> Option<BodyPartHandle>;

    /// The center of mass of this body part.
    fn center_of_mass(&self) -> Point<N>;

    /// The position of this body part wrt. the ground.
    fn position(&self) -> Isometry<N>;

    /// The velocity of this body part.
    fn velocity(&self) -> Velocity<N>;

    /// The world-space inertia of this body part.
    fn inertia(&self) -> Inertia<N>;

    /// The local-space inertia of this body part.
    fn local_inertia(&self) -> Inertia<N>;

    /// Apply a force to this body part at the next frame.
    fn apply_force(&mut self, force: &Force<N>);
}

downcast!(<N> Body<N> where N: Real);
downcast!(<N> BodyPart<N> where N: Real);