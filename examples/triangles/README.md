# Triangle

1. Pipeline Layout
    - bind_group_layouts
    - push_constant_ranges
2. Pipeline
    - shader
    - primitive
    - depth_stencil
    - multisample
    - multiview
3. vertex buffer / index buffer
    - Here, contents should be `&[u8]`, thus would need casting
4. Get Texture from Swapchain
5. Create commandEncoder
6. Begin renderpass and record some Cmds
    - `draw_*()`
        - draw
        - draw_indexed
    - `set_*()`
        - set_pipeline
        - set_{vertex/texture}_buffer

    ※ Here, renderpass `Drop` implicitly denotes the end of render pass
    ```rust
    {
        let must _render_pass = encoder.begin_render_pass()
        _render_pass.set_*();
        _render_pass.draw();
    }

    queue.submit(cmdBuffer)
    ```
7. Queue Submit
8. Present Texture
