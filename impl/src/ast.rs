use crate::attr::{self, Attrs};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Generics, Ident, Index, Member, Result,
    Type,
};

pub enum Input<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

pub struct Struct<'a> {
    pub attrs: Attrs,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

pub struct Enum<'a> {
    pub attrs: Attrs,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant<'a>>,
}

pub struct Variant<'a> {
    pub attrs: Attrs,
    pub ident: Ident,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub attrs: Attrs,
    pub member: Member,
    pub ty: &'a Type,
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a DeriveInput) -> Result<Self> {
        match &node.data {
            Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            Data::Enum(data) => Enum::from_syn(node, data).map(Input::Enum),
            Data::Union(_) => Err(Error::new_spanned(
                node,
                "union as errors are not supported",
            )),
        }
    }
}

impl<'a> Struct<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        Ok(Struct {
            attrs: attr::get(&node.attrs)?,
            ident: node.ident.clone(),
            generics: &node.generics,
            fields: Field::multiple_from_syn(&data.fields)?,
        })
    }
}

impl<'a> Enum<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        Ok(Enum {
            attrs: attr::get(&node.attrs)?,
            ident: node.ident.clone(),
            generics: &node.generics,
            variants: data
                .variants
                .iter()
                .map(Variant::from_syn)
                .collect::<Result<_>>()?,
        })
    }
}

impl<'a> Variant<'a> {
    fn from_syn(node: &'a syn::Variant) -> Result<Self> {
        Ok(Variant {
            attrs: attr::get(&node.attrs)?,
            ident: node.ident.clone(),
            fields: Field::multiple_from_syn(&node.fields)?,
        })
    }
}

impl<'a> Field<'a> {
    fn multiple_from_syn(fields: &'a Fields) -> Result<Vec<Self>> {
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::from_syn(i, field))
            .collect()
    }

    fn from_syn(i: usize, node: &'a syn::Field) -> Result<Self> {
        Ok(Field {
            attrs: attr::get(&node.attrs)?,
            member: node
                .ident
                .clone()
                .map(Member::Named)
                .unwrap_or_else(|| Member::Unnamed(Index::from(i))),
            ty: &node.ty,
        })
    }
}
