extern crate cgmath;
extern crate three;
//extern crate mint;

use self::cgmath::prelude::*;
use self::cgmath::{Matrix4, Rotation3, Transform as Transform_};
use self::cgmath::{Quaternion, Rad, Vector3};
use self::three::{object, Object};
use mint::Point3;

use self::three::controls::{Button, Input, MOUSE_LEFT};
// use node::TransformInternal;
// use three::Object;

/// Simple controls for Orbital Camera.
///
/// Camera is rotating around the fixed point with restrictions.
/// By default, it uses left mouse button as control button (hold it to rotate) and mouse wheel
/// to adjust distance to the central point.
#[derive(Clone, Debug)]
pub struct AlignedOrbit {
    pub object: object::Base,
    pub target: Point3<f32>,
    pub button: Button,
    pub speed: f32,
    pub xrot: f32,
    pub yrot: f32,
    pub distance: f32,
}

impl AlignedOrbit {
    pub fn new<T: Object>(object: &T) -> Self {
        Self {
            object: object.upcast(),
            target: [0.0, 0.0, 0.0].into(),
            button: MOUSE_LEFT,
            speed: 1.0,
            xrot: 0.0,
            yrot: 0.0,
            distance: 1.0,
        }
    }

    /// Update current position and rotation of the controlled object according to the last frame input.
    pub fn update(&mut self, input: &Input) {
        let mouse_delta = if input.hit(MOUSE_LEFT) {
            println!("mouse");
            input.mouse_delta_ndc()
        } else {
            [0.0, 0.0].into()
        };
        self.xrot = self.xrot + self.speed * mouse_delta.y;
        self.yrot = self.yrot + self.speed * mouse_delta.x;
        self.distance = self.distance + input.mouse_wheel() / 1000.0;

        let t = Matrix4::from_translation([0.0,0.0,-self.distance].into())        // move back self.distance
            * Matrix4::from_angle_x(Rad(self.yrot))         // rotate self.rotx around x
            * Matrix4::from_angle_y(Rad(self.xrot))        // rotate self.roty around y
            * Matrix4::from_translation([self.target.x,self.target.y,self.target.z].into()); // translate by self.target

        let p = t.transform_point([0.0, 0.0, 0.0].into());

        // println!("{:?}", [self.xrot, self.yrot, -1.0]);

        self.object.set_transform([self.xrot, self.yrot, -1.0], cgmath::Quaternion::from_angle_y(Rad(1.0)), 1.0);
        self.object.look_at([0.0, 0.0, 1.0], self.target, None);
    }
}
