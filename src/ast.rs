use std::{borrow::Cow, fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum NodeKind<'src, Anno> {
    Name {
        name: Cow<'src, str>,
    },
    App {
        fun: Rc<Node<'src, Anno>>,
        arg: Rc<Node<'src, Anno>>,
    },
    Abs {
        param: Rc<Node<'src, Anno>>,
        body: Rc<Node<'src, Anno>>,
    },
}

#[derive(Debug)]
pub struct Node<'src, Anno> {
    start: usize,
    end: usize,
    anno: Anno,
    kind: NodeKind<'src, Anno>,
}

#[derive(Default, Clone, Copy)]
pub struct ShowState {
    prio: usize,
}

pub trait Show {
    fn show(&self, st: &mut ShowState, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<'src, Anno> Show for Node<'src, Anno> {
    fn show(&self, st: &mut ShowState, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.show(st, f)
    }
}

impl<'src, Anno> Show for NodeKind<'src, Anno> {
    fn show(&self, st: &mut ShowState, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Name { name } => name.as_ref().fmt(f),
            NodeKind::App { fun, arg } => {
                fun.show(&mut ShowState{prio: st.prio + 1, ..*st}, f)?;
                " ".fmt(f)?;
                arg.show(st, f)
            }
            NodeKind::Abs { param, body } => {
                "\\ ".fmt(f)?;
                param.show(st, f)?;
                ". ".fmt(f)?;
                body.show(st, f)
            }
        }
    }
}
