//! This module defines traits for wrapping methods and async methods.
//! Should be brought into scope with `use crate::wrap::prelude::*;`.
//!
//! Allows for methods on userdata types defined in normal `impl` blocks and with `self` params
//! to be used as methods on Lua userdata types. Also allows for methods that don't take a Lua
//! context to be added without any wrapper functions.

use crate::prelude::*;
use std::future::Future;

// TODO: Figure out how to test these traits. They may not all be 100% correct.

pub trait FunctionNoLua<'lua, A, R>
where
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(A) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&Lua, A) -> mlua::Result<R> + 'static;
}
impl<'lua, F, A, R> FunctionNoLua<'lua, A, R> for F
where
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(A) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&Lua, A) -> mlua::Result<R> + 'static {
        move |_lua, args| self(args)
    }
}

pub trait FieldWithLua<'lua, T, R>
where
    T: 'static,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T, &Lua) -> mlua::Result<R> + 'static,
{
    fn wrap_field(self) -> impl Fn(&Lua, &T) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, R> FieldWithLua<'lua, T, R> for F
where
    T: 'static,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T, &Lua) -> mlua::Result<R> + 'static,
{
    fn wrap_field(self) -> impl Fn(&Lua, &T) -> mlua::Result<R> + 'static {
        move |lua, this| self(this, lua)
    }
}

pub trait FieldNoLua<'lua, T, R>
where
    T: 'static,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T) -> mlua::Result<R> + 'static,
{
    fn wrap_field(self) -> impl Fn(&Lua, &T) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, R> FieldNoLua<'lua, T, R> for F
where
    T: 'static,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T) -> mlua::Result<R> + 'static,
{
    fn wrap_field(self) -> impl Fn(&Lua, &T) -> mlua::Result<R> + 'static {
        move |_lua, this| self(this)
    }
}

pub trait MethodNoLuaNoArgs<'lua, T, R>
where
    T: 'static,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&Lua, &T, ()) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, R> MethodNoLuaNoArgs<'lua, T, R> for F
where
    T: 'static,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&Lua, &T, ()) -> mlua::Result<R> + 'static {
        move |_lua, this, _| self(this)
    }
}

/// Trait for wrapping async methods that take a Lua context.
pub trait AsyncMethodWithLua<'lua, 'a, T, A, R, MR>
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'a,
    Self: Sized + Fn(&'a T, &'lua Lua, A) -> MR + 'static,
{
    fn wrap_async(self) -> impl Fn(&'lua Lua, &'a T, A) -> MR;
}
impl<'lua, 'a, F, T, A, R, MR> AsyncMethodWithLua<'lua, 'a, T, A, R, MR> for F
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'a,
    Self: Sized + Fn(&'a T, &'lua Lua, A) -> MR + 'static,
{
    fn wrap_async(self) -> impl Fn(&'lua Lua, &'a T, A) -> MR {
        move |lua, this, args| self(this, lua, args)
    }
}

/// Trait for wrapping async methods that do not take a Lua context.
pub trait AsyncMethodNoLua<'lua, 'a, T, A, R, MR>
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'a,
    Self: Sized + Fn(&'a T, A) -> MR + 'static,
{
    fn wrap_async(self) -> impl Fn(&'lua Lua, &'a T, A) -> MR + 'static;
}
impl<'lua, 'a, F, T, A, R, MR> AsyncMethodNoLua<'lua, 'a, T, A, R, MR> for F
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'a,
    Self: Sized + Fn(&'a T, A) -> MR + 'static,
{
    fn wrap_async(self) -> impl Fn(&'lua Lua, &'a T, A) -> MR + 'static {
        move |_lua, this, args| self(this, args)
    }
}

/// Trait for wrapping async methods that take a Lua context and are mutable.
pub trait AsyncMethodMutWithLua<'lua, 'a, T, A, R, MR>
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'a,
    Self: Sized + Fn(&'a mut T, &'lua Lua, A) -> MR + 'static,
{
    fn wrap_async_mut(self) -> impl Fn(&'lua Lua, &'a mut T, A) -> MR;
}
impl<'lua, 'a, F, T, A, R, MR> AsyncMethodMutWithLua<'lua, 'a, T, A, R, MR> for F
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'a,
    Self: Sized + Fn(&'a mut T, &'lua Lua, A) -> MR + 'static,
{
    fn wrap_async_mut(self) -> impl Fn(&'lua Lua, &'a mut T, A) -> MR {
        move |lua, this, args| self(this, lua, args)
    }
}

/// Trait for wrapping async methods that do not take a Lua context and are mutable.
pub trait AsyncMethodMutNoLua<'lua, 'a, T, A, R, MR>
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'static,
    Self: Sized + Fn(&'a mut T, A) -> MR + 'static,
{
    fn wrap_async_mut(self) -> impl Fn(&'lua Lua, &'a mut T, A) -> MR;
}
impl<'lua, 'a, F, T, A, R, MR> AsyncMethodMutNoLua<'lua, 'a, T, A, R, MR> for F
where
    'lua: 'a,
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    MR: Future<Output = mlua::Result<R>> + 'static,
    Self: Sized + Fn(&'a mut T, A) -> MR + 'static,
{
    fn wrap_async_mut(self) -> impl Fn(&'lua Lua, &'a mut T, A) -> MR {
        move |_lua, this, args| self(this, args)
    }
}

/// Trait for wrapping methods that do not take a Lua context.
pub trait MethodNoLua<'lua, T, A, R>
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T, A) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&Lua, &T, A) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, A, R> MethodNoLua<'lua, T, A, R> for F
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T, A) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&Lua, &T, A) -> mlua::Result<R> + 'static {
        move |_lua, this, args| self(this, args)
    }
}

/// Trait for wrapping methods that take a Lua context.
pub trait MethodWithLua<'lua, T, A, R>
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T, &'lua Lua, A) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&'lua Lua, &T, A) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, A, R> MethodWithLua<'lua, T, A, R> for F
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&T, &'lua Lua, A) -> mlua::Result<R> + 'static,
{
    fn wrap(self) -> impl Fn(&'lua Lua, &T, A) -> mlua::Result<R> + 'static {
        move |lua, this, args| self(this, lua, args)
    }
}

/// Trait for wrapping methods that do not take a Lua context and are mutable.
pub trait MethodMutNoLua<'lua, T, A, R>
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&mut T, A) -> mlua::Result<R> + 'static,
{
    fn wrap_mut(self) -> impl Fn(&'lua Lua, &mut T, A) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, A, R> MethodMutNoLua<'lua, T, A, R> for F
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&mut T, A) -> mlua::Result<R> + 'static,
{
    fn wrap_mut(self) -> impl Fn(&'lua Lua, &mut T, A) -> mlua::Result<R> + 'static {
        move |_lua, this, args| self(this, args)
    }
}

/// Trait for wrapping methods that take a Lua context and are mutable.
pub trait MethodMutWithLua<'lua, T, A, R>
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&mut T, &Lua, A) -> mlua::Result<R> + 'static,
{
    fn wrap_mut(self) -> impl Fn(&Lua, &mut T, A) -> mlua::Result<R> + 'static;
}
impl<'lua, F, T, A, R> MethodMutWithLua<'lua, T, A, R> for F
where
    T: 'static,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
    Self: Sized + Fn(&mut T, &Lua, A) -> mlua::Result<R> + 'static,
{
    fn wrap_mut(self) -> impl Fn(&Lua, &mut T, A) -> mlua::Result<R> + 'static {
        move |lua, this, args| self(this, lua, args)
    }
}
