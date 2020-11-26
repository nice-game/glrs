use crate::vertex::VertexArray;
use gl::types::GLuint;
use std::ffi::c_void;

pub trait CommandBufferAbstract<C> {
	fn vao(&self) -> &VertexArray;
	fn handle(&self) -> GLuint;
	fn len(&self) -> usize;
	fn indirect(&self) -> *const c_void;
}

pub struct CommandBuffer<'a, C> {
	vao: &'a VertexArray,
	cmds: Vec<C>,
}
impl<'a, C> CommandBuffer<'a, C> {
	pub fn new(vao: &'a VertexArray) -> Self {
		Self { vao, cmds: vec![] }
	}
}
impl<'a> CommandBuffer<'a, DrawElementsIndirectCommand> {
	pub fn push(&mut self, count: u32, instance_count: u32, first_index: u32, base_vertex: u32, base_instance: u32) {
		self.cmds.push(DrawElementsIndirectCommand { count, instance_count, first_index, base_vertex, base_instance })
	}
}
impl<'a> CommandBuffer<'a, DrawArraysIndirectCommand> {
	pub fn push(&mut self, count: u32, instance_count: u32, base_vertex: u32, base_instance: u32) {
		self.cmds.push(DrawArraysIndirectCommand { count, instance_count, base_vertex, base_instance })
	}
}
impl<'a, C> CommandBufferAbstract<C> for CommandBuffer<'a, C> {
	fn vao(&self) -> &VertexArray {
		self.vao
	}

	fn handle(&self) -> GLuint {
		0
	}

	fn len(&self) -> usize {
		self.cmds.len()
	}

	fn indirect(&self) -> *const c_void {
		self.cmds.as_ptr() as _
	}
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DrawArraysIndirectCommand {
	pub count: u32,
	pub instance_count: u32,
	pub base_vertex: u32,
	pub base_instance: u32,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DrawElementsIndirectCommand {
	pub count: u32,
	pub instance_count: u32,
	pub first_index: u32,
	pub base_vertex: u32,
	pub base_instance: u32,
}
