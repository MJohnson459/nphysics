#![allow(missing_docs)] // For downcast.

use downcast_rs::Downcast;
use na::{DVector, Real};

use crate::object::{BodyPartHandle, BodySet};
use crate::solver::{ConstraintSet, IntegrationParameters, NonlinearConstraintGenerator};

/// The handle of a consraint.
pub type ConstraintHandle = usize;

/// Trait implemented by joint that operate by generating constraints to restrict the relative motion of two body parts.
pub trait JointConstraint<N: Real>: NonlinearConstraintGenerator<N> + Downcast + Send + Sync {
    /// Return `true` if the constraint is active.
    ///
    /// Typically, a constraint is disable if it is between two sleeping bodies, or, between bodies without any degrees of freedom.
    fn is_active(&self, bodies: &BodySet<N>) -> bool {
        let (b1, b2) = self.anchors();
        let body1 = try_ret!(bodies.body(b1.0), false);
        let body2 = try_ret!(bodies.body(b2.0), false);

        let ndofs1 = body1.status_dependent_ndofs();
        let ndofs2 = body2.status_dependent_ndofs();

        (ndofs1 != 0 && body1.is_active()) || (ndofs2 != 0 && body2.is_active())
    }

    /// The maximum number of velocity constraints generated by this joint.
    fn num_velocity_constraints(&self) -> usize;
    /// The two body parts affected by this joint.
    fn anchors(&self) -> (BodyPartHandle, BodyPartHandle);
    /// Initialize and retrieve all the constraints appied to the bodies attached to this joint.
    fn velocity_constraints(
        &mut self,
        params: &IntegrationParameters<N>,
        bodies: &BodySet<N>,
        ext_vels: &DVector<N>,
        ground_j_id: &mut usize,
        j_id: &mut usize,
        jacobians: &mut [N],
        velocity_constraints: &mut ConstraintSet<N>,
    );
    /// Called after velocity constraint resolution, allows the joint to keep a cache of impulses generated for each constraint.
    fn cache_impulses(&mut self, constraints: &ConstraintSet<N>);
}

impl_downcast!(JointConstraint<N> where N: Real);
