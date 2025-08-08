use std::ops::{Deref, DerefMut};
use structmeta::ToTokens;
use syn::{
    Path, Result, Token, Type, WherePredicate,
    parse::{Parse, ParseStream, discouraged::Speculative},
    parse_quote,
};

#[derive(Clone, ToTokens)]
pub enum Bound {
    Type(Type),
    Pred(WherePredicate),
    Default(Token![..]),
}

impl Parse for Bound {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![..]) {
            return Ok(Self::Default(input.parse()?));
        }
        let fork = input.fork();
        match fork.parse() {
            Ok(p) => {
                input.advance_to(&fork);
                Ok(Self::Pred(p))
            }
            Err(e) => {
                if let Ok(ty) = input.parse() {
                    Ok(Self::Type(ty))
                } else {
                    Err(e)
                }
            }
        }
    }
}

pub struct Bounds {
    pub ty: Vec<Type>,
    pub pred: Vec<WherePredicate>,
    pub can_extend: bool,
}
impl Bounds {
    fn new(can_extend: bool) -> Self {
        Bounds {
            ty: Vec::new(),
            pred: Vec::new(),
            can_extend,
        }
    }
    pub fn from_data(bound: Option<Vec<Bound>>) -> Self {
        if let Some(bound) = bound {
            let mut bs = Self::new(false);
            for b in bound {
                bs.push(b);
            }
            bs
        } else {
            Self::new(true)
        }
    }
    fn push(&mut self, bound: Bound) {
        match bound {
            Bound::Type(ty) => self.ty.push(ty),
            Bound::Pred(pred) => self.pred.push(pred),
            Bound::Default(_) => self.can_extend = true,
        }
    }
    pub fn child(&mut self, bounds: Option<Vec<Bound>>) -> BoundsChild<'_> {
        let bounds = if self.can_extend {
            Self::from_data(bounds)
        } else {
            Self::new(false)
        };
        BoundsChild {
            owner: self,
            bounds,
        }
    }
    pub fn build_wheres(self, trait_path: &Path) -> Vec<WherePredicate> {
        let mut pred = self.pred;
        for ty in self.ty {
            pred.push(parse_quote!(#ty : #trait_path));
        }
        pred
    }
}
pub struct BoundsChild<'a> {
    owner: &'a mut Bounds,
    bounds: Bounds,
}
impl Deref for BoundsChild<'_> {
    type Target = Bounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}
impl DerefMut for BoundsChild<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bounds
    }
}
impl Drop for BoundsChild<'_> {
    fn drop(&mut self) {
        if self.owner.can_extend {
            self.owner.ty.append(&mut self.bounds.ty);
            self.owner.pred.append(&mut self.bounds.pred);
        }
    }
}
