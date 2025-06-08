WebGPU is an API for more general GPU access. Firefox needs the
`dom.webgpu.enabled` flag to use it in the latest version. (I have enabled this
flag myself).

We shouldn't need WebGPU.

The error is coming from the wgpu library.

I think the problem is that we're asking for webgpu stuff but wgpu has
webgl2 limits? Even on chromium or with the flag enabled, the `AdaptorInfo` from
bevy-renderer says WebGL 2.0

Error seems to start at

```
<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_bind_group_layout
wgpu::api::device::Device::create_bind_group_layout
<bevy_jam6::GameOfLifePipeline as bevy_ecs::world::FromWorld>::from_world
```

So the problem comes from the RenderDevice resource.

SupApps have their own worlds. This is useful for e.g. a render thread.

Okay, you need to use the webgpu feature in the crate instead of the webgl2 feature.
