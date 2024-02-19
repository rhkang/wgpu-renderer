# Uniform
### Uniform Buffer
- Buffer Usage Type = `UNIFORM`
  - to write some data in runtime so that the shader can use updated data, denote `COPY_DST` here
- Type of binding = `Buffer` - `Uniform`
- Bind Group Resource Type = Buffer
  - give binding view here (e.g. `buffer.as_entire_buffer_binding()`)

Rest of works are same with texture binding case, just
1. bind the binding group layout to pipeline layout
2. bind the binding group to renderpass before submitting

#### Recall: MVP Transformation
- model: model coordinate -> (model Transformation: rotate or something)
- view: correspond to `lookAt`, change objects' transform with respect to camera's view
- proj: apply frustum-into-cube transformation to all objects
  - perspective: choose znear, zfar, fovy, aspect to determine the frustum
  - orthographic: let frustum has the same near, far planes

**Integrated**: proj * view * model (4x4 for homogeneous coord. system)

### Camera
- By modifying `aspect` in constructing the view frustum, you can get the same ratio between x, y coordinates we denote in code
- wgpu using z-coord in range (0, 1), thus if the current coordinate system(e.g. cgmath) works on OpenGL-way, map (-1, 1) into (0, 1)

```rust
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.5,
  0.0, 0.0, 0.0, 1.0,
);
```
