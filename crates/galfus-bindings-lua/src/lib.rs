use mlua::prelude::*;

fn galfus_init(_: &Lua, _: ()) -> LuaResult<u32> {
    Ok(galfus_core::galfus_init() as u32)
}

fn galfus_dispose(_: &Lua, _: ()) -> LuaResult<u32> {
    Ok(galfus_core::galfus_dispose() as u32)
}

fn galfus_send_queue(_: &Lua, data: LuaString) -> LuaResult<u32> {
    let bytes = data.as_bytes();
    Ok(galfus_core::galfus_send_queue(bytes.as_ptr(), bytes.len()) as u32)
}

fn take_string_result<F>(lua: &Lua, receive_fn: F) -> LuaResult<(LuaString, u32)>
where
    F: FnOnce(*mut *const u8, *mut usize) -> u32,
{
    let mut length: usize = 0;
    let mut ptr: *const u8 = std::ptr::null();
    let length_ptr = &mut length as *mut usize;
    let ptr_ptr = &mut ptr as *mut *const u8;

    let result = receive_fn(ptr_ptr, length_ptr);
    if result != 0 || length == 0 {
        return Ok((lua.create_string(&[])?, result));
    }

    let boxed = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
    let lua_string = lua.create_string(&boxed)?;
    Ok((lua_string, result))
}

fn galfus_receive_queue(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
    take_string_result(lua, |out_ptr, out_length| {
        galfus_core::galfus_receive_queue(out_ptr, out_length) as u32
    })
}

fn galfus_receive_events(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
    take_string_result(lua, |out_ptr, out_length| {
        galfus_core::galfus_receive_events(out_ptr, out_length) as u32
    })
}

fn galfus_upload_buffer(_: &Lua, (id, upload_type, data): (i64, u32, LuaString)) -> LuaResult<u32> {
    let bytes = data.as_bytes();
    Ok(
        galfus_core::galfus_upload_buffer(id as u64, upload_type, bytes.as_ptr(), bytes.len())
            as u32,
    )
}

fn galfus_tick(_: &Lua, (time, delta_time): (i64, u32)) -> LuaResult<u32> {
    Ok(galfus_core::galfus_tick(time as u64, delta_time) as u32)
}

fn galfus_get_profiling(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
    take_string_result(lua, |out_ptr, out_length| {
        galfus_core::galfus_get_profiling(out_ptr, out_length) as u32
    })
}

#[mlua::lua_module]
pub fn galfus(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("init", lua.create_function(galfus_init)?)?;
    exports.set("dispose", lua.create_function(galfus_dispose)?)?;
    exports.set("send_queue", lua.create_function(galfus_send_queue)?)?;
    exports.set("receive_queue", lua.create_function(galfus_receive_queue)?)?;
    exports.set(
        "receive_events",
        lua.create_function(galfus_receive_events)?,
    )?;
    exports.set("upload_buffer", lua.create_function(galfus_upload_buffer)?)?;
    exports.set("tick", lua.create_function(galfus_tick)?)?;
    exports.set("get_profiling", lua.create_function(galfus_get_profiling)?)?;
    Ok(exports)
}
