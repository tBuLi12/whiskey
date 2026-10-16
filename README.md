### Whiskey engine

Vulkan-based game engine. Very much in the prototyping phase.

The only backend is Vulkan, on all platforms. Support is planned only for Linux Wayland and Windows.

Modern Vulkan is assumed, at least 1.4, and support for VK_EXT_descriptor_heap and VK_EXT_shader_object is assumed to be mandatory, since by the time this engine becomes anything useful those things are planned to be widely adopted and available.

Mesh shaders are optional for now, since Intel doesn't seem very interested in supporting them.

The plan is to have the engine split into two main layers - the rendering layer and the game layer. Prototyping starts with the rendering layer, because the game layer will sit above it. However, it should be possible to access the rendering layer directly from the game layer.

The rendering layer wraps the platform's windowing/io APIs along with Vulkan and other stuff a game might need, like audio.

The point of having custom wrappers for the two platforms instead of using some existing windowing library is:
  - libraries assume a broader set of platforms to support, limiting the capabilities they expose.
  - libraries typically expose a common subset of capabilities of the platforms they support. If it's ever necessary, the rendering layer can expose some platform-specific functionality to the engine directly.

As for Vulkan, the rendering layer should generally expose interfaces such that someone familiar with the Vulkan API can fairly intuitively use them. However, the intention is not to expose all of Vulkan or all of its flexibility. Some assumptions are made, sometimes because this rendering layer targets fewer devices than Vulkan is expected to run on (eg. no mobile support), but sometimes just for simplicity.

The rendering layer handles presentation "manually", that is not through the Vulkan WSI. That's because the WSI is known to mess things up, the Vulkan-Docs repo has a lot of fun, unresolved issues with existential questions about which functions from the WSI API should block and when, and what that means exactly. Doing all of the talking to Wayland on Linux makes things much more transparent, at the cost of extra plumbing. On Windows the Vulkan WSI is known to be a second-class citizen, since DirectX is the primary API. That's why games like Doom Eternal I believe chose to present over a DXGI Swapchain instead, and that's what's done here too. Sharing a DXGI Swapchain image with Vulkan is a bit of a gray zone, but it seems to work okay, except for getting the semi-explicit sync of DX12 to work with the fully explicit sync of Vulkan, which was a bit tricky and seems to require scheduling at least one image-modifying command to a DX12 queue.

The main issue currently is what the API of the rendering layer should look like exactly. It makes sense to have an `App` and a way to `run` it, and then to be able to open windows with that. Each window can have some kind of object assigned which decides how this window behaves - handles input events for it and draws into it on the rendering layer's request. The rendering layer should probably also keep track of things like frames in flight, so it just handles creation of swapchain images and picking them, and just hands the per-window object an image to draw on. Still, this behavior should be at least somewhat configurable, number of frames in flight, maybe pipelining graphics/compute operations, and then it gets a little hard with multithreaded rendering. On the Vulkan side, wrapping primitives can get tricky at times, but generally follows straightforward patterns. The biggest issue is where these two API surfaces meet, which is when the drawing logic needs to be implemented. There's nothing inherently wrong with drawing directly with the exposed Vulkan primitives, but problems appear when modularization comes into the picture. Splitting drawing logic gets tricky, because what Vulkan expects is a transposed version of what the programmer wants to write. It's nice to write "use the text module to draw text, then use the shadows module to render shadows", but Vulkan expects "here are all upload operations for each module, then barriers, then here are all drawing commands for each module". The way this is typically solved, as far as I know, is by using render graphs, where each module adds its nodes and then the graph just sort of makes it work and schedules the necessary Vulkan commands. However, the API is supposed the wrap Vulkan fairly thinly, and exposing a render graph is not thin by any means, and render graphs, while useful in most game engines, aren't an absolute necessity for a rendering layer in general. So it would be nice to find some intermediate solution, some API that would help with code organization and decoupling when interacting with the rendering layer, but without bringing in all of the heavy machinery of a render graph.

Render layer is becoming quite a mouthful after spelling it out so many times, maybe it needs a better name.

### running

Validation layers are currently required to run, but they should be put behind a feature flag.
