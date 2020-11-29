use crate::{
	alloc::{Allocator, AllocatorAbstract},
	buffer::{BufferSlice, DynamicBuffer},
	Ctx,
};
use gl::types::{GLenum, GLint, GLuint};
use nalgebra::{allocator::Allocator as NAllocator, DefaultAllocator, Dim, DimName, Quaternion, Scalar, Unit, VectorN};
use simba::simd::SimdValue;
use std::{
	cell::{Cell, RefCell},
	mem::size_of,
	rc::Rc,
};

#[macro_export]
macro_rules! implement_vertex {
	($struct:ident, $($field:ident),+) => {
		impl $crate::vertex::Vertex for $struct {
			fn format() -> Vec<$crate::vertex::VertexAttributeFormat> {
				fn glformat<T: $crate::vertex::VertexAttribute>(_: Option<&T>)
					-> ($crate::gl::types::GLint, $crate::gl::types::GLenum)
				{
					(<T as $crate::vertex::VertexAttribute>::size(), <T as $crate::vertex::VertexAttribute>::typ())
				}

				vec![ $( {
					let offset = $crate::memoffset::offset_of!($struct, $field) as _;
					let (size, typ) = glformat(None::<&$struct>.map(|x| &x.$field));
					$crate::vertex::VertexAttributeFormat { offset, size, typ }
				} ),+ ]
			}
		}
	};
}

macro_rules! implement_attribute {
	($ty:ty, $size:expr, $typ:expr) => {
		impl VertexAttribute for $ty {
			fn size() -> GLint {
				$size
			}

			fn typ() -> GLenum {
				$typ
			}
		}
	};
}

pub trait Vertex {
	fn format() -> Vec<VertexAttributeFormat>;
}

pub trait VertexAttribute {
	fn size() -> GLint;
	fn typ() -> GLenum;
}
implement_attribute!(u8, 1, gl::UNSIGNED_BYTE);
implement_attribute!(u32, 1, gl::UNSIGNED_INT);
implement_attribute!(f32, 1, gl::FLOAT);
impl<N: Scalar + VertexAttribute, D: Dim + DimName> VertexAttribute for VectorN<N, D>
where
	DefaultAllocator: NAllocator<N, D>,
{
	fn size() -> GLint {
		D::try_to_usize().unwrap() as _
	}

	fn typ() -> GLenum {
		N::typ()
	}
}
impl<N: Scalar + SimdValue + VertexAttribute> VertexAttribute for Quaternion<N> {
	fn size() -> GLint {
		4
	}

	fn typ() -> GLenum {
		N::typ()
	}
}
impl<T: VertexAttribute> VertexAttribute for Unit<T> {
	fn size() -> GLint {
		T::size()
	}

	fn typ() -> GLenum {
		T::typ()
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct VertexAttributeFormat {
	pub offset: GLuint,
	pub size: GLint,
	pub typ: GLenum,
}

pub struct VertexArray {
	ctx: Rc<Ctx>,
	handle: GLuint,
	formats: RefCell<Vec<Vec<VertexAttributeFormat>>>,
	next_attrib: Cell<GLuint>,
	element_buffer: RefCell<Option<Rc<DynamicBuffer<[u8]>>>>,
	vertex_buffers: RefCell<Vec<Option<Rc<DynamicBuffer<[u8]>>>>>,
}
impl VertexArray {
	pub fn new(ctx: &Rc<Ctx>) -> Self {
		let mut handle = 0;
		unsafe { ctx.gl.CreateVertexArrays(1, &mut handle) };
		Self {
			ctx: ctx.clone(),
			handle,
			formats: vec![].into(),
			next_attrib: 0.into(),
			element_buffer: None.into(),
			vertex_buffers: vec![].into(),
		}
	}

	pub fn enable_vertices<V: Vertex>(&self, divisor: GLuint) {
		let format = V::format();
		let gl = &self.ctx.gl;
		for &VertexAttributeFormat { offset, size, typ } in &format {
			let next_attrib = self.next_attrib.get();
			let formats_len = self.formats.borrow().len() as _;
			unsafe {
				gl.EnableVertexArrayAttrib(self.handle, next_attrib);
				gl.VertexArrayAttribFormat(self.handle, next_attrib, size, typ, gl::FALSE, offset);
				gl.VertexArrayAttribBinding(self.handle, next_attrib, formats_len);
				gl.VertexArrayBindingDivisor(self.handle, formats_len, divisor);
			}
			self.next_attrib.set(next_attrib + 1);
		}
		self.formats.borrow_mut().push(format);

		self.vertex_buffers.borrow_mut().push(None);
	}

	pub fn element_buffer(&self, element_buffer: &Allocator<u16>) {
		let element_buffer = element_buffer.buffer();
		unsafe { self.ctx.gl.VertexArrayElementBuffer(self.handle, element_buffer.handle()) };
		*self.element_buffer.borrow_mut() = Some(element_buffer.clone());
	}

	pub fn vertex_buffer<V: Vertex>(&self, binding: usize, vertex_buffer: &Allocator<V>) {
		assert_eq!(V::format(), self.formats.borrow()[binding]);

		let vertex_buffer = vertex_buffer.buffer();
		let stride = size_of::<V>() as _;
		unsafe { self.ctx.gl.VertexArrayVertexBuffer(self.handle, binding as _, vertex_buffer.handle(), 0, stride) };
		self.vertex_buffers.borrow_mut()[binding] = Some(vertex_buffer.clone());
	}

	pub fn handle(&self) -> GLuint {
		self.handle
	}

	pub fn ctx(&self) -> &Rc<Ctx> {
		&self.ctx
	}
}
impl Drop for VertexArray {
	fn drop(&mut self) {
		unsafe { self.ctx.gl.DeleteVertexArrays(1, &self.handle) };
	}
}
