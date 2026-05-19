use super::super::RenderState;
use super::super::light::FrustumPlane;
use crate::core::resources::geometry::Frustum;

impl RenderState {
    pub(crate) fn prepare_lights(&mut self, _device: &wgpu::Device) {
        self.light_prepare_sorted_ids.clear();
        self.light_prepare_sorted_ids
            .extend(self.scene.lights.keys().copied());
        self.light_prepare_sorted_ids.sort_unstable();

        self.light_prepare_lights.clear();
        self.light_prepare_lights
            .reserve(self.light_prepare_sorted_ids.len());
        let mut shadow_counter = 0u32;
        for light_id in self.light_prepare_sorted_ids.iter().copied() {
            let Some(record) = self.scene.lights.get(&light_id) else {
                continue;
            };
            let mut light_data = record.data;
            if record.cast_shadow {
                light_data.shadow_index = shadow_counter;
                shadow_counter += 1;
            } else {
                light_data.shadow_index = 0xFFFFFFFF;
            }
            self.light_prepare_lights.push(light_data);
        }
        let light_count = self.light_prepare_lights.len();

        let light_system = match self.light_system.as_mut() {
            Some(sys) => sys,
            None => return,
        };

        if !self.light_prepare_lights.is_empty() {
            light_system
                .lights
                .write_slice(0, &self.light_prepare_lights);
        }
        light_system.light_count = light_count;

        self.light_prepare_frustums.clear();
        self.light_prepare_frustums
            .reserve(self.camera_order.len().saturating_mul(6));
        for camera_id in self.camera_order.iter().copied() {
            let Some(camera_record) = self.scene.cameras.get(&camera_id) else {
                continue;
            };
            let frustum = Frustum::from_view_projection(camera_record.data.view_projection);
            self.light_prepare_frustums.push(FrustumPlane {
                data: frustum.planes[0],
            });
            self.light_prepare_frustums.push(FrustumPlane {
                data: frustum.planes[1],
            });
            self.light_prepare_frustums.push(FrustumPlane {
                data: frustum.planes[2],
            });
            self.light_prepare_frustums.push(FrustumPlane {
                data: frustum.planes[3],
            });
            self.light_prepare_frustums.push(FrustumPlane {
                data: frustum.planes[4],
            });
            self.light_prepare_frustums.push(FrustumPlane {
                data: frustum.planes[5],
            });
        }

        if !self.light_prepare_frustums.is_empty() {
            light_system
                .camera_frustums
                .write_slice(0, &self.light_prepare_frustums);
        }
        light_system.camera_count = self.camera_order.len() as u32;

        let max_lights = 128; // TBD: make configurable or dynamic
        light_system.max_lights_per_camera = max_lights;

        let params = [
            light_count as u32,
            light_system.camera_count,
            max_lights,
            0, // Padding
        ];

        let params_buffer = light_system.params_buffer.as_ref().unwrap();
        light_system
            .queue
            .write_buffer(params_buffer, 0, bytemuck::cast_slice(&params));
    }
}
