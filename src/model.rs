use cgmath::{Matrix4, Rad, Vector3};


pub struct Model {
    position: Vector3<f32>,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
    roll: Rad<f32>,
    matrix: Matrix4<f32>,
}

impl Model {
    pub fn new<
            V: Into<Vector3<f32>>,
            T: Into<Rad<f32>>,
        >
        (position: V, pitch: T, yaw: T, roll: T) -> Self{
        let position = position.into();
        let pitch = pitch.into();
        let yaw = yaw.into();
        let roll = roll.into();
        let rot_x = Matrix4::from_angle_x(pitch);
        let rot_y = Matrix4::from_angle_y(yaw);
        let rot_z = Matrix4::from_angle_z(roll);
        // Extrinsic rotation (hopefully), left most happens first?
        let rotation = rot_x * rot_y * rot_z;
        let matrix = rotation * Matrix4::from_translation(position);
        Model {
            position,
            pitch,
            yaw,
            roll,
            matrix, 
        }
    }

    pub fn get_transformation_matrix(&self) -> [[f32; 4]; 4] {
        self.matrix.into()
    }

    pub fn rotate_z_extrinsic<
        V: Into<Rad<f32>>,
        >    
        (&mut self, rotation: V) {
        self.roll += rotation.into();
        let rot_x = Matrix4::from_angle_x(self.pitch);
        let rot_y = Matrix4::from_angle_y(self.yaw);
        let rot_z = Matrix4::from_angle_z(self.roll);
        // Extrinsic rotation (hopefully), left most happens first?
        let rotation = rot_x * rot_y * rot_z;
        let matrix = rotation * Matrix4::from_translation(self.position);
        self.matrix = matrix;
    }
}