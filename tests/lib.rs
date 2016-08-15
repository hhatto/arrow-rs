#![feature(cstr_from_bytes)]
#![feature(heap_api)]
#![feature(alloc)]

extern crate libc;
extern crate arrow;
extern crate alloc;

#[cfg(test)]
mod tests {
  use std::ffi::{CString, CStr};
  use std::ptr;
  use std::fs;
  use std::fs::File;
  use std::slice;
  use alloc::heap;

  use arrow::buffer;
  use arrow::column;
  use arrow::table;
  use arrow::ty;
  use arrow::array;
  use arrow::ipc::memory;
  use arrow::ipc::adapter;
  use arrow::types::primitive;
  use arrow::common::memory_pool;
  use arrow::common::status;

  #[test]
  fn test_field() {
    unsafe {
      let dt = ty::new_primitive_type(ty::Ty::INT32);
      assert_eq!(4, ty::value_size(dt));
      assert_eq!(CStr::from_bytes_with_nul(b"int32\0").unwrap(),
        CStr::from_ptr(ty::data_type_to_string(dt)));

      let dt2 = ty::new_primitive_type(ty::Ty::INT32);
      assert!(ty::data_type_equals(dt, dt2));

      let fp = ty::new_field(CString::new("f0").unwrap().as_ptr(), dt, false);
      assert_eq!(CStr::from_bytes_with_nul(b"f0: int32 not null\0").unwrap(),
        CStr::from_ptr(ty::field_to_string(fp)));

      let fp2 = ty::new_field(CString::new("f0").unwrap().as_ptr(), dt2, false);
      assert!(ty::field_equals(fp, fp2));

      let fields = [fp, fp2];
      let struct_field = ty::new_struct_type(2, &fields);
      assert_eq!(CStr::from_bytes_with_nul(b"struct<f0: int32, f0: int32>\0").unwrap(),
        CStr::from_ptr(ty::data_type_to_string(struct_field)));

      ty::release_field(fp);
      ty::release_field(fp2);
      ty::release_data_type(dt);
      ty::release_data_type(dt2);
    }
  }

  #[test]
  fn test_schema() {
    unsafe {
      let int_type = ty::new_primitive_type(ty::Ty::INT32);
      let float_type = ty::new_primitive_type(ty::Ty::FLOAT);
      let string_type = ty::new_string_type();

      let f0 = ty::new_field(CString::new("f0").unwrap().as_ptr(), int_type, false);
      let f1 = ty::new_field(CString::new("f1").unwrap().as_ptr(), float_type, false);
      let f2 = ty::new_field(CString::new("f2").unwrap().as_ptr(), string_type, false);
      let fields = [f0, f1, f2];


      let s = ty::new_schema(3, &fields);
      ty::release_schema(s);

      ty::release_field(f0);
      ty::release_field(f1);
      ty::release_field(f2);
      ty::release_data_type(string_type);
      ty::release_data_type(float_type);
      ty::release_data_type(int_type);
    }
  }

  #[test]
  fn test_buffer_resize() {

    unsafe {
      let pool = memory_pool::default_mem_pool();
      let buf_builder = buffer::new_buf_builder(pool);
      let val: u8 = 10;

      let s = buffer::raw_append_buf_builder(buf_builder, &val, 1);
      assert!(status::ok(s));
      status::release_status(s);

      let s = buffer::resize_buf_builder(buf_builder, 100);
      assert!(status::ok(s));
      status::release_status(s);

      assert_eq!(1, buffer::buf_builder_len(buf_builder));
      assert_eq!(128, buffer::buf_builder_capa(buf_builder));

      let buf = buffer::finish_buf_builder(buf_builder);
      assert_eq!(100, buffer::buf_size(buf));

      let s = buffer::resize_buf(buf, 50);
      assert!(status::ok(s));
      assert_eq!(50, buffer::buf_size(buf));
      assert_eq!(128, buffer::buf_capa(buf));

      buffer::release_buf(buf);
      buffer::release_buf_builder(buf_builder);
    }
  }

  #[test]
  fn test_array() {

    unsafe {
      // FIXME: using the single memory pool makes difficult to verify the amount of allocated memory
      let pool = memory_pool::default_mem_pool();
      let mem_before = memory_pool::num_bytes_alloc(pool);

      let uint8 = ty::new_primitive_type(ty::Ty::UINT8);
      let builder = primitive::new_u8_arr_builder(pool, uint8);
      let values: Vec<u8> = (0..32).collect();

      let s = primitive::append_u8_arr_builder(builder, values.as_ptr(), 32, ptr::null());
      assert!(status::ok(s));
      status::release_status(s);

      let arr = primitive::finish_u8_arr_builder(builder);

      let u8_ty = ty::new_primitive_type(ty::Ty::UINT8);
      assert!(ty::data_type_equals(u8_ty, array::arr_type(arr)));
      ty::release_data_type(u8_ty);

      assert_eq!(32, array::arr_len(arr));

      for i in 0..32 {
        assert_eq!(i as u8, primitive::u8_arr_value(arr, i));
      }

      array::release_arr(arr);

      assert_eq!(mem_before, memory_pool::num_bytes_alloc(pool));
    }
  }

  #[test]
  fn test_column() {

    unsafe {
      let pool = memory_pool::default_mem_pool();
      let f32_ty = ty::new_primitive_type(ty::Ty::FLOAT);
      let f1 = ty::new_field(CString::new("f1").unwrap().as_ptr(), f32_ty, false);
      let values: Vec<f32> = (0..32).map(|i| i as f32).collect();
      let builder = primitive::new_f32_arr_builder(pool, f32_ty);

      let s = primitive::append_f32_arr_builder(builder, values.as_ptr(), 32, ptr::null());
      assert!(status::ok(s));
      status::release_status(s);

      let arr = primitive::finish_f32_arr_builder(builder);
      assert_eq!(32, array::arr_len(arr));

      let col = column::new_column_from_arr(f1, arr);
      assert_eq!(32, column::column_len(col));
      assert_eq!(0, column::column_null_count(col));
      assert!(ty::data_type_equals(f32_ty, column::column_type(col)));
      let s = column::validate_column_data(col);
      assert!(status::ok(s));
      status::release_status(s);

      column::release_column(col);

      array::release_arr(arr);
      ty::release_field(f1);
      ty::release_data_type(f32_ty);
    }
  }

  #[test]
  fn test_row_batch() {
    unsafe {
      let pool = memory_pool::default_mem_pool();
      let f32_ty = ty::new_primitive_type(ty::Ty::FLOAT);
      let f1 = ty::new_field(CString::new("f1").unwrap().as_ptr(), f32_ty, false);
      let fields = [f1];
      let schema = ty::new_schema(1, &fields);
      let values: Vec<f32> = (0..32).map(|i| i as f32).collect();

      let builder = primitive::new_f32_arr_builder(pool, f32_ty);
      let s = primitive::append_f32_arr_builder(builder, values.as_ptr(), 32, ptr::null());
      status::release_status(s);
      let arrs = [primitive::finish_f32_arr_builder(builder)];

      let row_batch = table::new_row_batch(schema, 32, &arrs, 1);

      assert!(ty::schema_equals(schema, table::row_batch_schema(row_batch)));
      assert!(array::arr_equals(arrs[0], table::row_batch_column(row_batch, 0)));
      assert_eq!(32, table::row_batch_num_rows(row_batch));
      assert_eq!(1, table::row_batch_num_cols(row_batch));

      table::release_row_batch(row_batch);
      array::release_arr(arrs[0]);
      ty::release_schema(schema);
      ty::release_field(f1);
      ty::release_data_type(f32_ty);
    }
  }

  #[test]
  fn test_table() {
    unsafe {
      let pool = memory_pool::default_mem_pool();
      let f32_ty = ty::new_primitive_type(ty::Ty::FLOAT);
      let f1 = ty::new_field(CString::new("f1").unwrap().as_ptr(), f32_ty, false);
      let fields = [f1];
      let schema = ty::new_schema(1, &fields);
      let values: Vec<f32> = (0..32).map(|i| i as f32).collect();

      let builder = primitive::new_f32_arr_builder(pool, f32_ty);
      let s = primitive::append_f32_arr_builder(builder, values.as_ptr(), 32, ptr::null());
      status::release_status(s);
      let arrs = [primitive::finish_f32_arr_builder(builder)];
      let cols = [column::new_column_from_arr(f1, arrs[0])];

      let table = table::new_table(CString::new("t1").unwrap().as_ptr(), schema, &cols, 1);
      assert!(ty::schema_equals(schema, table::table_schema(table)));
      assert_eq!(1, table::table_num_cols(table));
      assert_eq!(32, table::table_num_rows(table));
      //      assert!(column::column_equals(cols[0], table::table_column(table, 0)));
      let s = table::validate_table_cols(table);
      assert!(status::ok(s));
      status::release_status(s);

      table::release_table(table);
      column::release_column(cols[0]);
      array::release_arr(arrs[0]);
      ty::release_schema(schema);
      ty::release_field(f1);
      ty::release_data_type(f32_ty);
    }
  }

  #[test]
  fn test_mem_src() {
    let mut f = File::create("test_mem_src.dat").unwrap();
    f.set_len(32).unwrap();
    f.sync_all().unwrap();

    unsafe {
      let src = memory::open_mmap_src(CString::new("test_mem_src.dat").unwrap().as_ptr(),
                                      memory::AccessMode::READ_WRITE);
      let values: Vec<u8> = (0..32).collect();
      let origin = values.clone();
      let s = memory::write_mmap_src(src, 0, values.as_ptr(), 32);
      assert!(status::ok(s));
      status::release_status(s);

      let s = memory::close_mmap_src(src);
      assert!(status::ok(s));
      status::release_status(s);
      memory::release_mmap_src(src);

      let src = memory::open_mmap_src(CString::new("test_mem_src.dat").unwrap().as_ptr(),
                                      memory::AccessMode::READ_ONLY);
      let buf = memory::read_at_mmap_src(src, 0, 32);
      let v = slice::from_raw_parts(buffer::buf_data(buf), 32);
      assert_eq!(&origin, &v);
      buffer::release_buf(buf);

      let s = memory::close_mmap_src(src);
      assert!(status::ok(s));
      status::release_status(s);
      memory::release_mmap_src(src);
    }

    fs::remove_file("test_mem_src.dat").unwrap();
  }

  #[test]
  fn test_adapter() {
    unsafe {
      let pool = memory_pool::default_mem_pool();
      let f32_ty = ty::new_primitive_type(ty::Ty::FLOAT);
      let f1 = ty::new_field(CString::new("f1").unwrap().as_ptr(), f32_ty, false);
      let fields = [f1];
      let schema = ty::new_schema(1, &fields);
      let values: Vec<f32> = (0..32).map(|i| i as f32).collect();

      let builder = primitive::new_f32_arr_builder(pool, f32_ty);
      let s = primitive::append_f32_arr_builder(builder, values.as_ptr(), 32, ptr::null());
      status::release_status(s);
      let arrs = [primitive::finish_f32_arr_builder(builder)];

      let row_batch = table::new_row_batch(schema, 32, &arrs, 1);

      let batch_size = adapter::c_api::get_row_batch_size(row_batch);

      let mut f = File::create("test_adapter.dat").unwrap();
      f.set_len(batch_size as u64).unwrap();
      f.sync_all().unwrap();

      let src = memory::open_mmap_src(CString::new("test_adapter.dat").unwrap().as_ptr(),
                                      memory::AccessMode::READ_WRITE);
      let header_pos = adapter::c_api::write_row_batch(src, row_batch, 0, 64);

      let s = memory::close_mmap_src(src);
      assert!(status::ok(s));
      status::release_status(s);
      memory::release_mmap_src(src);
      table::release_row_batch(row_batch);

      let src = memory::open_mmap_src(CString::new("test_adapter.dat").unwrap().as_ptr(),
                                      memory::AccessMode::READ_ONLY);

      let reader = adapter::c_api::open_row_batch_reader(src, header_pos);
      let row_batch = adapter::c_api::get_row_batch(reader, schema);

      let col = table::row_batch_column(row_batch, 0);
      assert!(array::arr_equals(arrs[0], col));

      let s = memory::close_mmap_src(src);
      assert!(status::ok(s));
      status::release_status(s);
      memory::release_mmap_src(src);

      adapter::c_api::release_row_batch_reader(reader);
      table::release_row_batch(row_batch);

      array::release_arr(arrs[0]);
      ty::release_schema(schema);
      ty::release_field(f1);
      ty::release_data_type(f32_ty);
    }

    fs::remove_file("test_adapter.dat").unwrap();
  }
}
