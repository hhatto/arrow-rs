#include "ty.h"

bool is_primitive_type(Type::type ty) {
  switch (ty) {
    case Type::NA:
    case Type::BOOL:
    case Type::UINT8:
    case Type::INT8:
    case Type::UINT16:
    case Type::INT16:
    case Type::UINT32:
    case Type::INT32:
    case Type::UINT64:
    case Type::INT64:
    case Type::FLOAT:
    case Type::DOUBLE:
      return true;
    default: {
      return false;
    }
  }
}

DataTypeBox* new_primitive_type(Type::type ty) {

  std::shared_ptr<DataType> sp;
  switch (ty) {
    case Type::NA: {
      sp = std::make_shared<NullType>();
      break;
    }
    case Type::BOOL: {
      sp = std::make_shared<BooleanType>();
      break;
    }
    case Type::UINT8: {
      sp = std::make_shared<UInt8Type>();
      break;
    }
    case Type::INT8: {
      sp = std::make_shared<Int8Type>();
      break;
    }
    case Type::UINT16: {
      sp = std::make_shared<UInt16Type>();
      break;
    }
    case Type::INT16: {
      sp = std::make_shared<Int16Type>();
      break;
    }
    case Type::UINT32: {
      sp = std::make_shared<UInt32Type>();
      break;
    }
    case Type::INT32: {
      sp = std::make_shared<Int32Type>();
      break;
    }
    case Type::UINT64: {
      sp = std::make_shared<UInt64Type>();
      break;
    }
    case Type::INT64: {
      sp = std::make_shared<Int64Type>();
      break;
    }
    case Type::FLOAT: {
      sp = std::make_shared<FloatType>();
      break;
    }
    case Type::DOUBLE: {
      sp = std::make_shared<DoubleType>();
      break;
    }
    default: {
      return nullptr;
    }
  }

  DataTypeBox* dt = new DataTypeBox;
  dt->sp = sp;
  dt->dt = sp.get();
  return dt;
}

DataTypeBox* new_list_type(DataTypeBox* value_type) {
  DataTypeBox* box = new DataTypeBox;
  box->sp = std::make_shared<ListType>(value_type->sp);
  box->dt = box->sp.get();
  return box;
}

DataTypeBox* new_binary_type() {
  DataTypeBox* box = new DataTypeBox;
  box->sp = std::make_shared<BinaryType>();
  box->dt = box->sp.get();
  return box;
}

DataTypeBox* new_string_type() {
  DataTypeBox* box = new DataTypeBox;
  box->sp = std::make_shared<StringType>();
  box->dt = box->sp.get();
  return box;
}

DataTypeBox* new_struct_type(int field_num, FieldBox* fields []) {
  std::vector<std::shared_ptr<Field>> vec;
  for (int i = 0; i < field_num; i++) {
    vec.push_back(fields[i]->sp);
  }
  DataTypeBox* box = new DataTypeBox;
  box->sp = std::make_shared<StructType>(vec);
  box->dt = box->sp.get();
  return box;
}
bool data_type_equals(const DataTypeBox* dt1, const DataTypeBox* dt2) {
  return dt1->dt->Equals(dt2->dt);
}

int value_size(DataTypeBox* dt) {
  return dt->dt->value_size();
}

const char* data_type_to_string(DataTypeBox* dt) {
  return dt->dt->ToString().c_str();
}

void release_data_type(DataTypeBox * dt) {
  if (dt) {
    delete dt;
  }
}

FieldBox* new_field(char* name, DataTypeBox* data_type, bool nullable) {
  FieldBox* fp = new FieldBox;
  fp->sp = std::make_shared<Field>(std::string(name), data_type->sp, nullable);
  fp->field = fp->sp.get();
  return fp;
}

bool field_equals(const FieldBox* f1, const FieldBox* f2) {
  return f1->field->Equals(*(f2->field));
}

const char* field_to_string(FieldBox* fp) {
  return fp->field->ToString().c_str();
}

void release_field(FieldBox* fp) {
  if (fp) {
    delete fp;
  }
}

SchemaBox* new_schema(int field_num, FieldBox* fields []) {
  std::vector<std::shared_ptr<Field>> vec;
  for (int i = 0; i < field_num; i++) {
    vec.push_back(fields[i]->sp);
  }

  SchemaBox* box = new SchemaBox;
  box->sp = std::make_shared<Schema>(vec);
  box->schema = box->sp.get();
  return box;
}

bool schema_equals(SchemaBox* s1, SchemaBox* s2) {
  return s1->schema->Equals(*(s2->schema));
}

const char* schema_to_string(SchemaBox* schema) {
  return schema->schema->ToString().c_str();
}

void release_schema(SchemaBox* schema) {
  if (schema) {
    delete schema;
  }
}