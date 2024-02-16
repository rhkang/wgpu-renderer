# Texture
1. Load Image
2. Create Texture
    - size
    - mip_level_count
    - sample_count
    - dimension
    - format
    - usage
        - `TEXTURE_BINDING` : use this texture in shaders
        - `COPY_DST` : want to copy data to this texture
    - view_formats

3. Write Image into Texture
4. Now use the data in Texture as a dedicated form, TextureView and Samplers
    - TextureView : a view into the texture
    - Sampler : controls how the texture is sampled
        - address_mode_{u/v/m} : handle the coordinate outside of texture
            - `ClampToEdge`
            - `Repeat`
            - `MirroredRepeat`
        - {[mag/min/mipmap](https://zumrakavafoglu.github.io/files/bca611-cg/lecture12/cg-lecture12.pdf)}_filter : when one texel grid is sparser or not than the texture pixel
            - `Linear` : select two texels in each dimension and return linear interpolation between them
            - `Nearest` : return the texel value nearest to the coordinate

5. Create BindGroup to describle how to access a set of resources for shaders
    - BindGroup
        - layout: `BindGroupLayout`
        - entries: `&[BindGroupEntry]`

    - BindGroupLayout
        - entries: `&[BindGroupLayoutEntry]`

    - BindGroupLayoutEntry
        - binding
        - visibility : `VERTEX` | `FRAGMENT`
        - ty : binding type
        - count : ?

    - BindGroupEntry
        - binding
        - resource: `BindingResource` (TextureView, Buffer, Sampler, ...)

6. Add BindingGroup into Pipeline Layout
7. Set BindingGroup in RenderPass


> ⚠️ In WGPU, world coordinates have the y-axis pointing up, while texture coordinate have the y-axis pointing down