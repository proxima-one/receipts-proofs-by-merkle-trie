use std::fmt;

// Field represents a struct field.
struct Field {
    name: String,
    index: usize,
    exported: bool,
    field_type: Type,
    tag: String,
}

// Type represents the attributes of a Rust type.
struct Type {
    name: String,
    kind: reflect::Kind,
    is_encoder: bool,
    is_decoder: bool,
    elem: Option<Box<Type>>,
}

// NilKind is the RLP value encoded in place of nil pointers.
enum NilKind {
    NilKindString,
    NilKindList,
}

// Tags represents struct tags.
struct Tags {
    nil_kind: NilKind,
    nil_ok: bool,
    optional: bool,
    tail: bool,
    ignored: bool,
}

// TagError is raised for invalid struct tags.
struct TagError {
    struct_type: String,
    field: String,
    tag: String,
    err: String,
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let field = if !self.struct_type.is_empty() {
            format!("{}.{}", self.struct_type, self.field)
        } else {
            format!("field {}", self.field)
        };
        write!(
            f,
            "rlp: invalid struct tag {} for {} ({})",
            self.tag, field, self.err
        )
    }
}

impl Default for TagError {
    fn default() -> Self {
        TagError {
            struct_type: String::new(),
            field: String::new(),
            tag: String::new(),
            err: String::new(),
        }
    }
}

// DefaultNilValue determines whether a nil pointer to t encodes/decodes
// as an empty string or empty list.
impl Type {
    fn default_nil_value(&self) -> NilKind {
        let kind = self.kind;
        if is_uint(kind) || kind == reflect::Kind::String || kind == reflect::Kind::Bool {
            NilKind::NilKindString
        } else {
            NilKind::NilKindList
        }
    }
}

// ProcessFields filters the given struct fields, returning only fields
// that should be considered for encoding/decoding.
fn process_fields(all_fields: Vec<Field>) -> Result<(Vec<Field>, Vec<Tags>), TagError> {
    let last_public = last_public_field(&all_fields);

    let mut fields = Vec::new();
    let mut tags = Vec::new();
    for field in all_fields {
        if !field.exported {
            continue;
        }
        let ts = parse_tag(field.clone(), last_public)?;
        if ts.ignored {
            continue;
        }
        fields.push(field);
        tags.push(ts);
    }

    let mut any_optional = false;
    let mut first_optional_name = String::new();
    for (i, ts) in tags.iter().enumerate() {
        let name = fields[i].name.clone();
        if ts.optional || ts.tail {
            if !any_optional {
                first_optional_name = name.clone();
            }
            any_optional = true;
        } else {
            if any_optional {
                let msg = format!(
                    "must be optional because preceding field {} is optional",
                    first_optional_name
                );
                return Err(TagError {
                    field: name.clone(),
                    err: msg,
                    ..Default::default()
                });
            }
        }
    }

    Ok((fields, tags))
}

fn parse_tag(field: Field, last_public: usize) -> Result<Tags, TagError> {
    let name = field.name.clone();
    let tag = field.tag.clone();
    let mut ts = Tags {
        nil_kind: NilKind::NilKindList,
        nil_ok: false,
        optional: false,
        tail: false,
        ignored: false,
    };

    for t in tag.split(',') {
        match t.trim() {
            "" => {} // empty tag is allowed
            "-" => ts.ignored = true,
            "nil" => {
                ts.nil_ok = true;
                if field.field_type.kind != reflect::Kind::Ptr {
                    return Err(TagError {
                        field: name.clone(),
                        tag: t.to_string(),
                        err: "field is not a pointer".to_string(),
                        ..Default::default()
                    });
                }
                ts.nil_kind = field.field_type.elem.as_ref().unwrap().default_nil_value();
            }
            "nilString" => {
                ts.nil_ok = true;
                ts.nil_kind = NilKind::NilKindString;
            }
            "nilList" => {
                ts.nil_ok = true;
                ts.nil_kind = NilKind::NilKindList;
            }
            "optional" => {
                ts.optional = true;
                if ts.tail {
                    return Err(TagError {
                        field: name.clone(),
                        tag: t.to_string(),
                        err: "also has 'tail' tag".to_string(),
                        ..Default::default()
                    });
                }
            }
            "tail" => {
                ts.tail = true;
                if field.index != last_public {
                    return Err(TagError {
                        field: name.clone(),
                        tag: t.to_string(),
                        err: "must be on last field".to_string(),
                        ..Default::default()
                    });
                }
                if field.field_type.kind != reflect::Kind::Slice {
                    return Err(TagError {
                        field: name.clone(),
                        tag: t.to_string(),
                        err: "field type is not slice".to_string(),
                        ..Default::default()
                    });
                }
            }
            _ => {
                return Err(TagError {
                    field: name.clone(),
                    tag: t.to_string(),
                    err: "unknown tag".to_string(),
                    ..Default::default()
                });
            }
        }
    }

    Ok(ts)
}

fn last_public_field(fields: &[Field]) -> usize {
    let mut last = 0;
    for field in fields {
        if field.exported {
            last = field.index;
        }
    }
    last
}

fn is_uint(kind: reflect::Kind) -> bool {
    kind >= reflect::Kind::Uint && kind <= reflect::Kind::Uintptr
}

fn is_byte(typ: &Type) -> bool {
    typ.kind == reflect::Kind::Uint8 && !typ.is_encoder
}

fn is_byte_array(typ: &Type) -> bool {
    (typ.kind == reflect::Kind::Slice || typ.kind == reflect::Kind::Array) && is_byte(&*typ.elem.as_ref().unwrap())
}
