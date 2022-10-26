use std::{marker::PhantomData, mem::size_of, ops::Deref};

use bytemuck::{cast_slice, Pod};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferDescriptor, BufferUsages, Device, Queue,
};

pub trait Bufferable {
    const LABEL: &'static str;
}

impl Bufferable for u16 {
    const LABEL: &'static str = "IndexBuffer";
}

////////////////////////////////////////////////////////////////////////////////
// Buffer
////////////////////////////////////////////////////////////////////////////////

pub struct Buffer<T: Copy + Pod + Bufferable> {
    pub(super) buffer: wgpu::Buffer,
    length: usize,
    phantom: PhantomData<T>,
}

impl<T: Copy + Pod + Bufferable> Buffer<T> {
    pub fn new(device: &Device, data: &[T], usage: BufferUsages) -> Self {
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(T::LABEL),
                contents: cast_slice(data),
                usage,
            }),
            length: data.len(),
            phantom: PhantomData,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Dynamic Buffer
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DynamicBuffer<T: Copy + Pod + Bufferable>(Buffer<T>);

impl<T: Copy + Pod + Bufferable> DynamicBuffer<T> {
    pub fn new(device: &Device, length: usize, usage: BufferUsages) -> Self {
        Self(Buffer {
            buffer: device.create_buffer(&BufferDescriptor {
                label: Some(T::LABEL),
                size: size_of::<T>() as u64 * length as u64, // BUG
                usage: usage | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            length,
            phantom: PhantomData,
        })
    }

    /// Update GPU-size value
    pub fn update(&self, queue: &Queue, values: &[T], offset: usize) {
        if !values.is_empty() {
            queue.write_buffer(
                &self.buffer,
                offset as u64 * size_of::<T>() as u64,
                cast_slice(values),
            );
        }
    }
}

impl<T: Copy + Pod + Bufferable> Deref for DynamicBuffer<T> {
    type Target = Buffer<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Uniform Array Buffer
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A handle to a series of constants on the GPU.
pub struct Consts<T: Copy + Pod + Bufferable> {
    buffer: DynamicBuffer<T>,
}

impl<T: Copy + Pod + Bufferable> Consts<T> {
    pub fn new(device: &Device, length: usize) -> Self {
        Self {
            buffer: DynamicBuffer::new(device, length, BufferUsages::UNIFORM),
        }
    }

    pub fn update(&self, queue: &Queue, values: &[T], offset: usize) {
        self.buffer.update(queue, values, offset)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer.buffer
    }
}
