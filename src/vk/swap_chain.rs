use ash::vk;
use ash::vk::SurfaceFormatKHR;
use ash::vk::ValidationCacheCreateInfoEXT;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

use ash;

use std::os::raw::c_char;

use crate::vk::constants;
use crate::utility::constants as global_constants;
use crate::vk::debug;
use crate::utility::tools;

use crate::vk::render_device;
use crate::rhi::window;

pub struct VkSpawChain {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,

    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl VkSpawChain {
    pub fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: &render_device::VkSurface,
        queue_family: &render_device::QueueFamilyIndices
    ) -> VkSpawChain {
        let swapchain_support = VkSpawChain::query_swapchain_support(physical_device, &surface);

        let surface_format = VkSpawChain::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = VkSpawChain::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = VkSpawChain::choose_swapchain_extent(&swapchain_support.capabilities);

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        }
        else {
            image_count
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) = 
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2 as u32,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count: queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1
        };

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create Swapchain!")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };

        VkSpawChain {
            swapchain_loader,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
        }
    }

    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface: &render_device::VkSurface
    ) -> SwapChainSupportDetail {
        let capabilities = unsafe {
            surface
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface.surface)
            .expect("Failed to get physical device surface capabilities")
        };
        let formats = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface.surface)
                .expect("Failed to get defice surface formats")
        };
        let present_modes = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface.surface)
                .expect("Failed to get device surface present modes")
        };

        SwapChainSupportDetail {
            capabilities: capabilities,
            formats: formats,
            present_modes: present_modes
        }
    }

    fn choose_swapchain_format(
        available_formats: &Vec<ash::vk::SurfaceFormatKHR>
    ) -> ash::vk::SurfaceFormatKHR {

        for format in available_formats.iter() {
            if format.format == ash::vk::Format::B8G8R8A8_SRGB
                && format.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR {
                    return format.clone();
                }
        }

        return available_formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(
        present_modes: &Vec<ash::vk::PresentModeKHR>
    ) -> ash::vk::PresentModeKHR {

        for &present_mode in present_modes.iter() {
           if present_mode == ash::vk::PresentModeKHR::MAILBOX {
                return present_mode;
           }
        }

        return present_modes.first().unwrap().clone();
    }

    fn choose_swapchain_extent(
        capabilities: &ash::vk::SurfaceCapabilitiesKHR
    ) -> ash::vk::Extent2D {

        if capabilities.current_extent.width != u32::max_value() || capabilities.current_extent.height != u32::max_value() {
            capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D {
                width: clamp(
                    global_constants::WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    global_constants::WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}