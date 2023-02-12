use std::fmt::Debug;

use crate::ir::FieldValue;

use super::basic_adapter::{ContextIterator, ContextOutcomeIterator, VertexIterator};

/// Helper macro to produce function used by [`resolve_property_with`]
///
/// Retrieves a [FieldValue] from a vertex by converting it to the proper type,
/// and then retrieving the attribute
///
/// # Examples
/// ```
/// # use trustfall_core::interpreter::helpers::resolve_property_with;
/// struct User {
///     id: String
///     // ...
/// }
/// enum Token {
///     UserToken(Rc<User>)
///     // ...
/// }
/// impl Token {
///     pub fn as_user(&self) -> Option<&User> {
///         match self {
///             Token::UserToken(u) => Some(u.as_ref()),
///             _ => None,
///         }
///     }
///     // ...
/// }
///
/// // In implementation of `BasicAdapter`
/// fn resolve_property(
///     &mut self,
///     contexts: ContextIterator<'static, Self::Vertex>,
///     type_name: &str,
///     property_name: &str,
/// ) -> ContextOutcomeIterator<'static, Self::Vertex, FieldValue> {
///     match (type_name, property_name) {
///         ("User", "id") => resolve_property_with(contexts, property_resolve!(as_user, id)),
///         // ...
///         _ => uncreachable!()
///     }
/// ```
#[macro_export]
macro_rules! property_resolver {
    ($conversion:ident, $attr:ident) => {
        |vertex| -> FieldValue {
            let vertex = vertex.$conversion().expect("conversion failed");
            (&vertex.$attr).into()
        }
    };
    ($conversion:ident, $var:ident, $b:block) => {
        |vertex| -> FieldValue {
            let $var = vertex.$conversion().expect("conversion failed");
            $b
        }
    };
}

/// Helper for implementing [`BasicAdapter::resolve_property`] and equivalents.
///
/// Takes a property-resolver function and applies it over each of the vertices
/// in the input context iterator, one at a time. Often used with
/// [`property_resolver!`](trustfall_core::property_resolver)
pub fn resolve_property_with<'vertex, Vertex: Debug + Clone + 'vertex>(
    contexts: ContextIterator<'vertex, Vertex>,
    mut property_resolver: impl FnMut(&Vertex) -> FieldValue + 'static,
) -> ContextOutcomeIterator<'vertex, Vertex, FieldValue> {
    Box::new(contexts.map(move |ctx| match ctx.current_token.as_ref() {
        None => (ctx, FieldValue::Null),
        Some(vertex) => {
            let value = property_resolver(vertex);
            (ctx, value)
        }
    }))
}

/// Helper for implementing [`BasicAdapter::resolve_neighbors`] and equivalents.
///
/// Takes a neighbor-resolver function and applies it over each of the vertices
/// in the input context iterator, one at a time.
pub fn resolve_neighbors_with<'vertex, Vertex: Debug + Clone + 'vertex>(
    contexts: ContextIterator<'vertex, Vertex>,
    mut neighbors_resolver: impl FnMut(&Vertex) -> VertexIterator<'vertex, Vertex> + 'static,
) -> ContextOutcomeIterator<'vertex, Vertex, VertexIterator<'vertex, Vertex>> {
    Box::new(contexts.map(move |ctx| {
        match ctx.current_token.as_ref() {
            None => {
                // rustc needs a bit of help with the type inference here,
                // due to the Box<dyn Iterator> conversion.
                let no_neighbors: VertexIterator<'vertex, Vertex> = Box::new(std::iter::empty());
                (ctx, no_neighbors)
            }
            Some(vertex) => {
                let neighbors = neighbors_resolver(vertex);
                (ctx, neighbors)
            }
        }
    }))
}

/// Helper for implementing [`BasicAdapter::resolve_coercion`] and equivalents.
///
/// Takes a coercion-resolver function and applies it over each of the vertices
/// in the input context iterator, one at a time.
pub fn resolve_coercion_with<'vertex, Vertex: Debug + Clone + 'vertex>(
    contexts: ContextIterator<'vertex, Vertex>,
    mut coercion_resolver: impl FnMut(&Vertex) -> bool + 'static,
) -> ContextOutcomeIterator<'vertex, Vertex, bool> {
    Box::new(contexts.map(move |ctx| match ctx.current_token.as_ref() {
        None => (ctx, false),
        Some(vertex) => {
            let can_coerce = coercion_resolver(vertex);
            (ctx, can_coerce)
        }
    }))
}
