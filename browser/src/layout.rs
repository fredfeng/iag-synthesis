#![allow(unused_variables)]
#![allow(dead_code)]

///! Basic CSS block layout.
///
/// N.B.: The version of this file kept under version control is meant as a
/// "safe" fallback/baseline, omitting more recent improvements to the CSS
/// attribute grammar. Please don't check in each new auto-generated version,
/// especially while still debugging.

use crate::dom::DocumentNode;
use crate::style::{StyledTree, StyledNode, Style, DisplayType, Floated, Positioned, Overflow};
use crate::paint::DisplayList;
use crate::utility::{Pixels, Edge, Rect, FloatCursor, MarginAccumulator};
use crate::lazy::Lazy;
use std::fmt;
use itertools::Itertools;

const CASSIUS_LAYOUT_NAME: &str = "doc-2";

/// Construct the layout tree around to the style tree, returning it with all
/// layout constraints solved.
pub fn layout_tree<'a>(style_tree: &'a StyledTree<'a>, parameters: Parameters) -> LayoutTree<'a> {
    let mut layout_tree = LayoutTree::new(style_tree, parameters);
    layout_tree.layout();
    layout_tree
}

/// Fold the layout tree into a display list to render.
pub fn display_list(layout_tree: &LayoutTree) -> DisplayList {
    layout_tree.render()
}

/// Output parameters.
#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub scrollbar_width: usize,
    pub font_size: usize,
}

/// The full layout tree, with ownership of the composite layout nodes.
pub struct LayoutTree<'a> {
    style_tree: &'a StyledTree<'a>,
    parameters: Parameters,
    layout_root: LayoutNode<'a>
}

impl<'a> LayoutTree<'a> {
    fn new(style_tree: &'a StyledTree<'a>, parameters: Parameters) -> Self {
        LayoutTree {
            style_tree,
            parameters,
            layout_root: LayoutNode::new(&style_tree.style_root)
        }
    }
}

impl<'a> fmt::Display for LayoutTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head = format!(
            "({} :matched true :w {} :h {} :fs {} :scrollw {})",
            CASSIUS_LAYOUT_NAME,
            self.parameters.viewport_width, self.parameters.viewport_height,
            self.parameters.font_size, self.parameters.scrollbar_width
        );
        let body = format!(
            "([VIEW :w {}] {})",
            self.parameters.viewport_width,
            self.layout_root
        );
        write!(f, "(define-layout {} {})", head, body)
    }
}

/// Packaged layout data for each box.
#[derive(Clone, Default, Debug)]
pub struct Layout {
    /// Position and size of the containing block.
    containing_box: Rect<Pixels>,
    /// Position and size of the positioned containing block.
    positioning_box: Rect<Pixels>,
    /// Position and size of the content box relative to the document origin.
    content_box: Rect<Pixels>,
    /// Position and size of the padding box relative to the document origin.
    padding_box: Rect<Pixels>,
    /// Position and size of the border box relative to the document origin.
    border_box: Rect<Pixels>,
    /// Position and size of the margin box relative to the document origin.
    margin_box: Rect<Pixels>,
    /// Edges of the padding box.
    padding: Edge<Pixels>,
    /// Edges of the border box.
    border: Edge<Pixels>,
    /// Edges of the margin box.
    margin: Edge<Pixels>,
    /// Excess (or missing) horizontal space.
    underflow: Pixels,
    /// Cumulative positioning state for all predecessor floats.
    float_cursor: Lazy<FloatCursor>,
    /// Upper accumulator for vertical margin.
    upper_margin: MarginAccumulator,
    /// Lower accumulator for vertical margin.
    lower_margin: MarginAccumulator,
    /// Logical position in the block (vertical) axis.
    block_pos: Pixels,
    /// Logical position in the inline (horizontal) axis, before float shifting.
    inline_pos: Pixels,
    /// Logical position in the current stack of line boxes.
    line_pos: u32,
    /// Logical size in the block (vertical) axis, excluding out-of-flow content
    /// and collapsible margin.
    block_size: Pixels,
    /// Actual size in the block (vertical) axis, including out-of-flow content
    /// and collapsible margin.
    block_extent: Pixels,
    /// Logical size in the inline (horizontal) axis, excluding out-of-flow content.
    inline_size: Pixels,
    /// Actual size in the inline (horizontal) axis, including out-of-flow content.
    inline_extent: Pixels,
    ///  Carried margin in the block axis
    carried_margin: Pixels,

    /// actual margin used in other computations, equivalent to collapsed margin
    effective_margin: Edge<Pixels>,
    /// whether the element is self collapsing, default value is automaticall set to false
    self_collapse: bool,
    self_margin: Pixels,

    ns_positioning_box: Rect<Pixels>,
    init_positioning_box: Rect<Pixels>,
}

/// A node in the layout tree.
pub struct LayoutNode<'a> {
    document_node: Option<&'a DocumentNode>,
    /// Specified values from styling.
    style: &'a Style,
    /// Layout state for this node.
    layout: Layout,
    /// Fundamental layout mode (e.g., block, inline, float, absolute, &c.).
    class: LayoutClass,
    /// Zero or more descendant (child) boxes.
    children: Vec<LayoutNode<'a>>,
}

impl<'a> LayoutNode<'a> {
    fn new(style_node: &'a StyledNode) -> Self {
        LayoutNode::from_style_node(style_node).pop().unwrap()
    }

    /// Construct a new layout node at the block level.
    ///
    /// With this constructor, the caller signals that the resulting layout
    /// node is immediately inside a block container. After reaching an inline
    /// container, recursive construction switches to `new_at_inline_level` to
    /// permit
    ///
    /// A style node with a display type of "none" is omitted.
    fn from_style_node(style_node: &'a StyledNode) -> Vec<Self> {
        let style = &style_node.specified;
        let class = match LayoutClass::of_style_node(style_node) {
            None => { return Vec::new(); },
            Some(class) => class
        };
        let generate = |child_nodes| LayoutNode {
            document_node: Some(style_node.node),
            style,
            class,
            children: child_nodes,
            layout: Layout::default()
        };

        let mut child_iter =
            style_node.children
                .iter()
                .flat_map(LayoutNode::from_style_node)
                .peekable();

        // An inline container distributes itself over contiguous runs of
        // inline-level boxes, to effectively break around any transitively
        // contained in-flow block-level boxes. If needed, an empty split
        // of the inline container is added to cap outer block-level boxes.

        let mut contents = Vec::new();
        if class.is_block_container() {
            while child_iter.peek().is_some() { // (BlockLevel* InlineLevel*)*
                // First, greedily consume block-level children.
                contents.extend(
                    child_iter.peeking_take_while(LayoutNode::is_block_level)
                );
                // Check for termination eagerly to avoid empty anonymous
                // wrappers.
                if child_iter.peek().is_none() {
                    break;
                }
                // Once exhausted, greedily consume inline-level children for
                // anonymous wrapping (including intervening floated boxes).
                contents.push(LayoutNode::into_inline_root(
                    style,
                    child_iter.peeking_take_while(LayoutNode::is_inline_level)
                ));
            }

            // if class.is_floated() {
            // ==JUFIX== chrome810370: if no children, don't add any anon node
            if class.is_floated() && style_node.children.len()>0 {
                contents = vec![LayoutNode::into_block_root(style, contents)];
            }

            vec![generate(contents)]
        } else /* class.is_inline_container() */ {
            loop { // InlineLevel* (BlockFlow+ InlineLevel*)*
                // First, greedily consume inline-level children.
                contents.push(generate(
                    child_iter.peeking_take_while(LayoutNode::is_inline_level).collect()
                ));
                // Check for termination only when we have inline boxes capping
                // both ends.
                if child_iter.peek().is_none() {
                    break;
                }
                // Once exhausted, consume in-flow block-level children for
                // anonymous wrapping (excluding intervening floated boxes).
                contents.push(LayoutNode::into_block_container(
                    style,
                    child_iter.peeking_take_while(LayoutNode::is_block_flow)
                ));
            }

            contents
        }
    }

    fn into_inline_root<I: IntoIterator<Item=Self>>(parent_style: &'a Style, iterable: I) -> Self {
        let wrapped_children = iterable.into_iter().collect_vec();
        assert!(wrapped_children.iter().all(LayoutNode::is_inline_level));
        // println!("called into_inline_root");
        LayoutNode::anon(LayoutClass::InlineRoot, parent_style, wrapped_children)
    }

    fn into_block_root<I: IntoIterator<Item=Self>>(parent_style: &'a Style, iterable: I) -> Self {
        let wrapped_children = iterable.into_iter().collect_vec();
        assert!(wrapped_children.iter().all(LayoutNode::is_block_level));
        // println!("called into_block_root");
        LayoutNode::anon(LayoutClass::BlockRoot, parent_style, wrapped_children)
    }

    fn into_block_container<I: IntoIterator<Item=Self>>(parent_style: &'a Style, iterable: I) -> Self {
        let wrapped_children = iterable.into_iter().collect_vec();
        assert!(wrapped_children.iter().all(LayoutNode::is_block_level));
        // println!("called into_block_container");
        LayoutNode::anon(LayoutClass::Block, parent_style, wrapped_children)
    }

    /// Create an anonymous layout node wrapping a segment of nodes.
    fn anon(wrapper_class: LayoutClass, parent_style: &'a Style, wrapped_nodes: Vec<Self>) -> Self {
        LayoutNode {
            document_node: None,
            style: Box::leak(Box::new(Style::inherit(parent_style))),
            layout: Layout::default(),
            class: wrapper_class,
            children: wrapped_nodes
        }
    }

    fn is_block_level(&self) -> bool { self.class.is_block_level() }
    fn is_block_container(&self) -> bool { self.class.is_block_container() }
    fn is_block_root(&self) -> bool { self.class.is_block_root() }
    fn is_block_flow(&self) -> bool { self.is_block_level() && !self.is_floated() }

    fn is_inline_level(&self) -> bool { self.class.is_inline_level() }
    fn is_inline_container(&self) -> bool { self.class.is_inline_container() }
    fn is_inline_root(&self) -> bool { self.class.is_inline_root() }
    fn is_inline_flow(&self) -> bool { self.is_inline_level() && !self.is_floated() }

    fn is_floated(&self) -> bool { self.class.is_floated() }
    fn is_floated_left(&self) -> bool { self.is_floated() && self.style.float == Floated::Left }
    fn is_floated_right(&self) -> bool { self.is_floated() && self.style.float == Floated::Right }

    fn is_text_run(&self) -> bool { self.class.is_text_run() }

    fn is_positioned(&self) -> bool { self.style.position.is_positioned() }

    fn is_relative(&self) -> bool { self.style.position == Positioned::Relative }

    fn has_offsets(&self) -> bool { self.style.left.is_some() && self.style.right.is_some() && self.style.top.is_some() && self.style.bottom.is_some() }

    fn is_in_flow(&self) -> bool {
        use LayoutClass::*;
        use Positioned::*;

        match self.class {
            Text | Line | Inline | InlineRoot | InlineBlock | Block => true,
            Floated => false,
            BlockRoot => match self.style.position {
                Static | Relative => true,
                Absolute | Fixed | Sticky => false,
            },
        }
    }

    fn is_out_of_flow(&self) -> bool { !self.is_in_flow() }

    fn is_anon(&self) -> bool { self.document_node.is_none() }
}

impl<'a> fmt::Display for LayoutNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LayoutClass::*;

        let geometry = format!(
            ":x {} :y {} :w {} :h {}",
            self.layout.border_box.x,
            self.layout.border_box.y,
            self.layout.border_box.width,
            self.layout.border_box.height
        );
        let elem = self.document_node.map(|doc_node| doc_node.index);
        let text = self.document_node.and_then(DocumentNode::as_text);
        let header = match self.class {

            // ==JUFIX==: raw fix for automatically created line box
            // may not be complete
            _ if self.is_anon() && self.is_inline_root() =>
                String::from("[LINE]"),

            _ if self.is_anon() =>
                String::from("[ANON]"),
            
            Text =>
                format!("[TEXT {} :text \"{}\"]", geometry, text.unwrap()),
            Line =>
                String::from("[LINE]"),
            Inline | InlineRoot =>
                format!("[INLINE :elt {}]", elem.unwrap()),
            InlineBlock =>
                format!("[INLINE {} :elt {}]", geometry, elem.unwrap()),
            BlockRoot | Block | Floated =>
                format!("[BLOCK {} :elt {}]", geometry, elem.unwrap()),
        };

        f.write_str("(")?;
        f.write_str(&header)?;
        for child in self.children.iter() {
            write!(f, " {}", child)?;
        }
        f.write_str(")")?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutClass {
    Text,
    Line, // inline-level inline container
    Inline, // inline-level inline container
    InlineRoot, // block-level inline container
    InlineBlock, // inline-level block container
    Block, // block-level block container
    BlockRoot, // block-level block container
    Floated, // floated block container
}

impl LayoutClass {
    fn of_style_node(style_node: &StyledNode) -> Option<Self> {
        if style_node.as_text().is_some() {
            Some(LayoutClass::Text)
        } else if style_node.node.tag() == Some("html") {
            Some(LayoutClass::BlockRoot)
        } else {
            let style = &style_node.specified;
            if style.overflow != Overflow::Visible {
                Some(LayoutClass::BlockRoot)
                // Some(LayoutClass::Floated)
            } else {
                match style.position {
                    Positioned::Absolute | Positioned::Fixed =>
                        Some(LayoutClass::BlockRoot),
                    Positioned::Sticky =>
                        unimplemented!("sticky positioning unsupported"),
                    Positioned::Relative | Positioned::Static => match style.float {
                        Floated::Left | Floated::Right =>
                            Some(LayoutClass::Floated),
                        Floated::None => match style.display {
                            DisplayType::Block => Some(LayoutClass::Block),
                            DisplayType::Inline => Some(LayoutClass::Inline),
                            DisplayType::InlineBlock => Some(LayoutClass::InlineBlock),
                            DisplayType::None => None,
                        },
                    },
                }
            }
        }
    }

    /// Is this class of node a block-level box?
    fn is_block_level(&self) -> bool {
        match self {
            LayoutClass::Block => true, // block box
            LayoutClass::BlockRoot => true, // block root box
            LayoutClass::InlineRoot => true, // inline root box
            LayoutClass::Floated => true, // floated box with block anchor
            _ => false
        }
    }

    /// Is this class of node serve an inline-level box?
    fn is_inline_level(&self) -> bool {
        match self {
            LayoutClass::Text => true, // text run (with position metadata)
            LayoutClass::Line => true, // line box (needed?)
            LayoutClass::Inline => true, // inline box
            LayoutClass::InlineBlock => true, // inline-level block container box
            LayoutClass::Floated => true, // floated box with inline anchor
            _ => false
        }
    }

    /// Is this class of node a block container box?
    ///
    /// Note that a floated box implicitly wraps its children in a `BlockRoot`.
    fn is_block_container(&self) -> bool {
        match self {
            LayoutClass::Block => true, // block box
            LayoutClass::BlockRoot => true, // block root box
            LayoutClass::Floated => true, // floated box with block anchor
            LayoutClass::InlineBlock => true, // inline-level block container box
            _ => false
        }
    }

    /// Is this class of node an inline container box?
    fn is_inline_container(&self) -> bool {
        match self {
            LayoutClass::Line => true, // line box (necessary?)
            LayoutClass::Inline => true, // inline box
            LayoutClass::InlineRoot => true, // inline root box
            _ => false
        }
    }

    /// Is this class of node a block root box?
    ///
    /// N.B.: A floated box implicitly wraps its children under a `BlockRoot`.
    fn is_block_root(&self) -> bool {
        match self {
            LayoutClass::BlockRoot => true, // block root box
            LayoutClass::Floated => true, // floated box with block anchor
            LayoutClass::InlineBlock => true, // inline-level block container box
            _ => false
        }
    }

    /// Is this class of node an inline root box?
    ///
    /// N.B.: An inline root box is always an anonymous block-level box.
    fn is_inline_root(&self) -> bool {
        match self {
            LayoutClass::InlineRoot => true, // inline box
            _ => false
        }
    }

    // Is this class of node a floated box?
    fn is_floated(&self) -> bool {
        match self {
            LayoutClass::Floated => true,
            _ => false
        }
    }

    // Is this class of node a text run?
    fn is_text_run(&self) -> bool {
        match self {
            LayoutClass::Text => true,
            _ => false
        }
    }
}

impl<'a> LayoutTree<'a> {
    fn layout(&mut self) {
        let width = self.parameters.viewport_width as Pixels;
        let height = self.parameters.viewport_height as Pixels;
        let block = Rect { x: 0.0, y: 0.0, width: width, height: height };
        self.layout_root.layout.containing_box = block;
        self.layout_root.layout.positioning_box = block;
        self.layout_root.layout.ns_positioning_box = block;
        self.layout_root.layout.init_positioning_box = block;
        self.layout_root.layout.block_pos = 0.0;
        self.layout_root.layout.inline_pos = 0.0;
        self.layout_root.layout.line_pos = 0;
        // before layout, first compute the collapsed margin: effective margin
        self.layout_root.compute_effective_margin();
        self.layout_root.layout();
    }

    fn render(&self) -> DisplayList {
        let mut list = DisplayList::new();
        self.layout_root.render(&mut list);
        list
    }
}

impl<'a> LayoutNode<'a> {
    /// Lay out a box and its descendants.
    fn layout(&mut self) {
        match self.class {
            LayoutClass::InlineRoot => self.layout_inline_root(),
            LayoutClass::Inline => self.layout_inline(),
            LayoutClass::InlineBlock => self.layout_block(),
            LayoutClass::BlockRoot => self.layout_block(),
            LayoutClass::Block if !self.is_anon() => self.layout_block(),
            LayoutClass::Floated => self.layout_float(),
            _ => { },
        }
    }

    /// only root node can call
    /// compute collapsed margin for you
    fn compute_effective_margin(&mut self) {
        // println!("float: {}",self.is_floated());
        // println!("sc status: {}",self.layout.self_collapse);
        self.layout.margin.top = self.style.margin.top.value();
        self.layout.margin.bottom = self.style.margin.bottom.value();
        self.layout.margin.left = self.style.margin.left.value();
        self.layout.margin.right = self.style.margin.right.value();

        self.layout.effective_margin.left = self.layout.margin.left;
        self.layout.effective_margin.right = self.layout.margin.right;

        // println!("====");
        // println!("self.layout.margin.top: {}",self.layout.margin.top);
        // println!("self.layout.margin.bottom: {}",self.layout.margin.bottom);
        // println!("====");

        // compute sibling collapse in children
        let mut prev_child_in_flow: Option<&mut LayoutNode> = None;
        for child in &mut self.children {

            child.compute_effective_margin();
            // till here, all effective_margin fields are available

            if child.is_in_flow() && !child.is_block_root() {

                if let Some(p) = prev_child_in_flow {
                    // if you have a sibling also in flow, then try to collapse
                    // do some math: compute actual margin
                    let a = child.layout.effective_margin.top;
                    let b = p.layout.effective_margin.bottom;
                    let actual_margin = if a>=0.0 && b>=0.0 {
                        a.max(b)
                    } else if a<0.0 && b<0.0 {
                        a.min(b)
                    } else {
                        // one of them >0
                        a.max(b) + a.min(b)
                    };
                    child.layout.effective_margin.top = actual_margin;
                    p.layout.effective_margin.bottom = 0.0;
                }

                prev_child_in_flow = Some(child);
            }
            else {
                // immediately clear this since the neighbor should be strict
                prev_child_in_flow = None;
            }
        }

        // empty block self collapsing
        // FIXME

        // compute parent-child collapse
        if self.is_block_root() {
            // no need to collapse
            self.layout.effective_margin.top = self.layout.margin.top;
            self.layout.effective_margin.bottom = self.layout.margin.bottom;
        }
        else {

            // top margin collapse with first in-flow child
            if self.style.border.top==0.0 && self.style.padding.top==0.0 {
                // can do top margin collapse

                let mut has_first_child = false;
                for child in &mut self.children {
                    if child.is_floated() {
                        // it's blocking
                        break;
                        // if it's absolute, then it's not
                        // this condition may be imprecise
                    }
                    if child.is_in_flow() {
                        has_first_child = true;

                        // first in-flow child
                        // do some math: compute actual margin
                        let a = child.layout.effective_margin.top;
                        let b = self.layout.margin.top;
                        let actual_margin = if a>=0.0 && b>=0.0 {
                            a.max(b)
                        } else if a<0.0 && b<0.0 {
                            a.min(b)
                        } else {
                            // one of them >0
                            a.max(b) + a.min(b)
                        };
                        self.layout.effective_margin.top = actual_margin;
                        child.layout.effective_margin.top = 0.0;
                        break;
                    }
                    // strictly the very first
                    // break;
                }

                if !has_first_child {
                    self.layout.effective_margin.top = self.layout.margin.top;
                }
            }
            else {
                // blocked, no need to do top margin collapse
                self.layout.effective_margin.top = self.layout.margin.top;
            }

            // bottom margin collapse with last in-flow child
            if self.style.border.bottom==0.0 && self.style.padding.top==0.0 {
                // can do bottom margin collapse

                let mut has_last_child = false;
                self.children.reverse(); // =============== in-place =============== //
                for child in &mut self.children {
                    if child.is_floated() {
                        // it's blocking
                        break;
                        // if it's absolute, then it's not
                        // this condition may be imprecise
                    }
                    if child.is_in_flow() {
                        has_last_child = true;

                        // last in-flow child
                        // do some math: compute actual margin
                        let a = child.layout.effective_margin.bottom;
                        let b = self.layout.margin.bottom;
                        let actual_margin = if a>=0.0 && b>=0.0 {
                            a.max(b)
                        } else if a<0.0 && b<0.0 {
                            a.min(b)
                        } else {
                            // one of them >0
                            a.max(b) + a.min(b)
                        };
                        self.layout.effective_margin.bottom = actual_margin;
                        child.layout.effective_margin.bottom = 0.0;
                        break;
                    }
                    // strictly the very last
                    // break;
                }
                self.children.reverse(); // =============== in-place =============== //

                if !has_last_child {
                    self.layout.effective_margin.bottom = self.layout.margin.bottom;
                }
            }
            else {
                // blocked, no need to do bottom margin collapse
                self.layout.effective_margin.bottom = self.layout.margin.bottom;
            }
            
        }
        

        

    }


    /// Lay out a block-level element and its descendants.
    fn layout_inline_root(&mut self) {
        println!("call layout_inline_root");
        self.layout.padding = Edge::default();
        self.layout.border = Edge::default();

        // Position the box flush left (w.r.t. margin/border/padding) to the containing block.
        self.layout.content_box.x = self.layout.inline_pos;
        self.layout.content_box.width = self.layout.containing_box.width;

        // println!("inline root ---> content box width: {}",self.layout.content_box.width);

        // Position the box below all the previous boxes in the container.
        self.layout.content_box.y = self.layout.block_pos;
        self.layout.content_box.height = self.style.height.value();

        // println!("(bf) ir float cursor left: {}",self.layout.float_cursor.left_block_end);
        // println!("(bf) ir float cursor block start: {}",self.layout.float_cursor.block_start);

        // Recursively lay out the children of this box.
        let mut line_cursor = 0u32;
        let mut line_height = 0.0f32;
        // let mut line_height = 5.6f32;
        let mut block_cursor = self.layout.containing_box.y;
        let (mut inline_start, mut inline_end) = self.layout.float_cursor.inline_space(
            self.layout.containing_box.x,
            self.layout.containing_box.x + self.layout.containing_box.width,
            block_cursor
        );
        // println!("(af) ir float cursor left: {}",self.layout.float_cursor.left_block_end);
        // println!("(af) ir float cursor block start: {}",self.layout.float_cursor.block_start);
        // println!("ir float cursor right: {}",self.layout.float_cursor.right_block_end);
        // println!("ir containing box x: {}",self.layout.containing_box.x);
        // println!("ir containing box width: {}",self.layout.containing_box.width);
        // println!("ir block_cursor: {}",block_cursor);

        // ==JUFIX== QuickFix
        // check if there's float, if yes, no line height will be added
        let mut has_float = false;
        for child in &mut self.children {
            if child.class==LayoutClass::Floated {
                has_float = true;
                break;
            }
        }

        let mut inline_cursor = inline_start;
        for child in &mut self.children {
            // println!("ir inline cursor: {}",inline_cursor);
            // Give the child box the boundaries of its container.
            child.layout.containing_box = self.layout.content_box;
            child.layout.positioning_box = self.layout.positioning_box;
            child.layout.block_pos = block_cursor;
            child.layout.inline_pos = inline_cursor;
            child.layout.line_pos = line_cursor;
            child.layout.float_cursor = self.layout.float_cursor.clone();
            // Lay out the child box.
            child.layout();
            // Increment the cursor so each child is laid out below the previous one.
            if !child.is_positioned() {
                if inline_start < inline_cursor && inline_cursor + child.layout.inline_extent > inline_end {
                    block_cursor += line_height;
                    line_cursor += 1;
                    line_height = 0.0;
                    let p = self.layout.float_cursor.inline_space(
                        self.layout.containing_box.x,
                        self.layout.containing_box.x + self.layout.containing_box.width,
                        block_cursor
                    );
                    inline_start = p.0;
                    inline_end = p.1;
                    inline_cursor = inline_start;
                }
                inline_cursor += child.layout.inline_size;
                line_height = line_height.max(child.layout.block_size);
            }

            self.layout.block_extent = self.layout.block_extent.max(child.layout.block_extent);
            self.layout.float_cursor = child.layout.float_cursor.clone();

            // ==JUFIX== Quickfix
            // FIXME: only works when there's one element in the inline-root
            if child.class==LayoutClass::InlineBlock && !has_float{
                // println!("QF");
                block_cursor += 5.6;
            }

        }
        block_cursor += line_height;

        // Parent height can depend on child height, so `calculate_height` must be called after the
        // children are laid out.
        self.layout.content_box.height = if self.style.height.is_auto() {
            block_cursor - self.layout.content_box.y
        } else {
            self.style.height.value()
        };

        self.layout.padding_box = self.layout.content_box.extend_by(&self.layout.padding);
        self.layout.border_box = self.layout.padding_box.extend_by(&self.layout.border);
        self.layout.margin_box = self.layout.border_box.extend_by(&self.layout.effective_margin);

        self.layout.block_size = self.layout.margin_box.height.max(0.0);
    }

    /// Lay out a block-level element and its descendants.
    fn layout_inline(&mut self) {
        println!("call layout_inline");
        self.layout.padding = self.style.padding;
        self.layout.border = self.style.border;

        // Position the box flush left (w.r.t. margin/border/padding) to the containing block.
        self.layout.content_box.x = self.layout.inline_pos;
        // self.layout.content_box.width = self.style.width.value();
        // ==JUFIX== display:inline will nullify width and height
        self.layout.content_box.width = self.layout.containing_box.width;
        // println!("INLINE content box width: {}",self.layout.content_box.width);
        // println!("inline content box x: {}",self.layout.content_box.x);

        // Position the box below all the previous boxes in the container.
        // ==JUFIX== display:inline will nullify width and height
        self.layout.content_box.y = self.layout.block_pos;
        // self.layout.content_box.height = self.style.height.value();
        // self.layout.content_box.height = self.layout.containing_box.height;


        // Recursively lay out the children of this box.
        let block_cursor = self.layout.content_box.y;
        let mut block_size = 0.0f32;
        let mut inline_cursor = self.layout.content_box.x;
        let mut inline_size = 0.0f32;
        for child in &mut self.children {
            // println!("computed block_cursor: {}",block_cursor);
            // println!("computed inline_cursor: {}",inline_cursor);
            // Give the child box the boundaries of its container.
            child.layout.containing_box = self.layout.content_box;
            child.layout.positioning_box = self.layout.positioning_box;
            child.layout.block_pos = block_cursor;
            child.layout.inline_pos = inline_cursor;
            child.layout.line_pos = self.layout.line_pos;
            child.layout.float_cursor = self.layout.float_cursor.clone();
            // Lay out the child box.
            child.layout();
            // Increment the cursor so each child is laid out below the previous one.
            if !child.is_positioned() {
                // println!("increment");
                inline_cursor = inline_cursor + child.layout.inline_size;
                inline_size = inline_size + child.layout.inline_size;
                block_size = block_size.max(child.layout.block_size);
            }

            self.layout.block_extent = self.layout.block_extent.max(child.layout.block_extent);
            self.layout.float_cursor = child.layout.float_cursor.clone();
        }

        // Parent height can depend on child height, so `calculate_height` must be called after the
        // children are laid out.
        self.layout.content_box.height = if self.style.height.is_auto() {
            block_cursor - self.layout.content_box.y
        } else {
            self.style.height.value()
        };

        self.layout.padding_box = self.layout.content_box.extend_by(&self.layout.padding);
        self.layout.border_box = self.layout.padding_box.extend_by(&self.layout.border);
        self.layout.margin_box = self.layout.border_box.extend_by(&self.layout.effective_margin);

        self.layout.block_size = self.layout.margin_box.height.max(0.0);
    }

    /// Lay out a block-level element and its descendants.
    fn layout_block(&mut self) {
        println!("call layout_block");
        self.layout.padding = self.style.padding;
        self.layout.border = self.style.border;

        // Child width can depend on parent width, so we need to calculate this box's width before
        // laying out its children.
        self.calculate_block_width();

        if self.is_positioned() {
            self.layout.containing_box = self.layout.positioning_box;
        }

        let mut clearance: f32 = 0.0;
        let mut has_clear = false;
        if self.style.clear.left {
            // println!("clear left");
            has_clear = true;
            clearance = clearance.max(
                self.layout.float_cursor.left_clearance()
            );
        }
        if self.style.clear.right {
            // println!("clear right");
            has_clear = true;
            clearance = clearance.max(
                self.layout.float_cursor.right_clearance()
            );
        }

        // println!("block float cursor left: {}",self.layout.float_cursor.left_block_end);

        if self.style.overflow != Overflow::Visible || self.class==LayoutClass::InlineBlock {
            self.layout.content_box.width = if self.style.width.is_auto() {
                // min(max(preferred_minimum_width, available_width), preferred_width)
                self.layout.containing_box.width
                - self.layout.padding.left
                - self.layout.padding.right
                - self.layout.border.left
                - self.layout.border.right
                - self.layout.effective_margin.left
                - self.layout.effective_margin.right
                // - self.layout.margin.left
                // - self.layout.margin.right
            } else {
                self.style.width.value()
            };
            self.layout.content_box.height = self.style.height.value();

            let mut available = self.layout.containing_box.clone();
            available.y = self.layout.block_pos;
            let outer_width =
                self.layout.content_box.width
                + self.layout.padding.left
                + self.layout.padding.right
                + self.layout.border.left
                + self.layout.border.right
                + self.layout.effective_margin.left
                + self.layout.effective_margin.right;
                // + self.layout.margin.left
                // + self.layout.margin.right;
            let (inline, block) = self.layout.float_cursor.place_left(&available, outer_width);

            self.layout.content_box.x =
                inline
                + self.layout.padding.left
                + self.layout.border.left
                + self.layout.effective_margin.left;
                // + self.layout.margin.left;
            self.layout.content_box.y =
                block
                + self.layout.padding.top
                + self.layout.border.top
                + self.layout.effective_margin.top;
                // + self.layout.margin.top;

            // FIXME (servo3456): borrowed from layout_float to deal with overflow position with float
            // may need to correct some other variables
        }
        else {
            // Position the box flush left (w.r.t. margin/border/padding) to the container.
            self.layout.content_box.x =
                self.layout.inline_pos
                + self.layout.padding.left
                + self.layout.border.left
                + self.layout.effective_margin.left;

            // Position the box below all the previous boxes in the container.
            
            self.layout.content_box.y =
                self.layout.block_pos
                + self.layout.padding.top
                + self.layout.border.top
                + self.layout.effective_margin.top;

            // if self.class==LayoutClass::InlineBlock {
            //     println!("YEAH");
            //     println!("inline pos: {}",self.layout.inline_pos);
            //     println!("content box x: {}",self.layout.content_box.x);
            // }
        }

        



        if has_clear {
            // ==JUFIX== //
            // SPIT OUT the effective margin: clearance inhibits margin
            // https://stackoverflow.com/questions/4198269/margin-top-not-working-with-clear-both/41335816#41335816
            // this solution may not be complete
            self.layout.content_box.y -= self.layout.effective_margin.top;

            // println!("block_pos: {}",self.layout.block_pos);
            // println!("self.layout.effective_margin.top: {}",self.layout.effective_margin.top);
            // println!("init content_box.y: {}",self.layout.content_box.y);
            self.layout.content_box.y = self.layout.content_box.y.max(clearance);
            // println!("clear content_box.y: {}",self.layout.content_box.y);
        }
        
        

        self.layout.content_box.height = self.style.height.value();

        if self.is_positioned() {
            // println!("start is_positioned");
            if let Some(abs_left) = self.style.left {
                // println!("b1");
                self.layout.content_box.x =
                    self.layout.positioning_box.x
                    + abs_left
                    + self.layout.padding.left
                    + self.layout.border.left
                    + self.layout.margin.left;
            } else if let Some(abs_right) = self.style.right {
                self.layout.content_box.x =
                    self.layout.positioning_box.x
                    + self.layout.positioning_box.width
                    - self.layout.content_box.width
                    - abs_right;
            }
            // ==JUFIX:3== comment out
            // else {
            //     self.layout.content_box.width = 0.0;
            // }
            if let Some(abs_top) = self.style.top {
                // println!("b1");
                self.layout.content_box.y =
                    self.layout.positioning_box.y
                    + abs_top;
            } else if let Some(abs_bottom) = self.style.bottom {
                self.layout.content_box.y =
                    self.layout.positioning_box.y
                    + self.layout.positioning_box.height
                    - self.layout.content_box.height
                    - abs_bottom;
            }
            // println!("computed self.layout.positioning_box.x: {}", self.layout.positioning_box.x);
            // println!("computed self.layout.positioning_box.y: {}", self.layout.positioning_box.y);
            // println!("computed self.layout.content_box.x: {}", self.layout.content_box.x);
            // println!("computed self.layout.content_box.y: {}", self.layout.content_box.y);
        }

        if self.is_relative() {
            if let Some(dx) = self.style.left {
                self.layout.content_box.x += dx;
            }
            if let Some(dy) = self.style.top {
                self.layout.content_box.y += dy;
            }
        }
        // println!("middle content_box.y: {}",self.layout.content_box.y);

        // Recursively lay out the children of this box.
        let mut block_cursor = self.layout.content_box.y;
        let pre_border_box = self.layout.content_box.extend_by(&self.layout.padding).extend_by(&self.layout.border);
        for child in &mut self.children {
            // println!("??? child float: {}",child.is_floated());
            // Give the child box the boundaries of its container.
            child.layout.containing_box = self.layout.content_box;
            // child.layout.positioning_box = if self.style.position == Positioned::Static {
            //     self.layout.positioning_box
            // } else {
            //     self.layout.containing_box
            // };

            child.layout.init_positioning_box = self.layout.init_positioning_box;
            child.layout.ns_positioning_box = if self.style.position == Positioned::Static {
                self.layout.ns_positioning_box
            }
            else {
                // self.layout.content_box
                pre_border_box
            };
            child.layout.positioning_box = if child.style.position == Positioned::Absolute {
                child.layout.ns_positioning_box
            } else if child.style.position == Positioned::Fixed {
                child.layout.init_positioning_box
            } else {
                // self.layout.content_box
                pre_border_box
            };

            child.layout.inline_pos = self.layout.content_box.x;
            child.layout.block_pos = block_cursor;
            child.layout.float_cursor = self.layout.float_cursor.clone();

            // Lay out the child box.
            child.layout();
            // Increment the cursor so each child is laid out below the previous one.
            if child.is_in_flow() {
                block_cursor += child.layout.block_size;
                // println!("block_cursor: {}",block_cursor);
            }
            // ==JUFIX==: should think of clearance div that extends the parent height
            // this is an override of the previous condition
            if child.style.clear.left || child.style.clear.right {
                // then directly set the jumped block cursor
                block_cursor = child.layout.content_box.y 
                             + child.layout.content_box.height 
                             + child.layout.padding.bottom
                             + child.layout.border.bottom
                             + child.layout.effective_margin.bottom;
                // println!("reset block_cursor: {}",block_cursor);

            }
            // println!("== effective_margin bottom: {}",child.layout.effective_margin.bottom);

            self.layout.block_extent = self.layout.block_extent.max(child.layout.block_extent);
            self.layout.float_cursor = child.layout.float_cursor.clone();
            
        }

        // ==JUFIX== QuickFix
        if !self.style.height.is_auto() {
            if self.style.overflow == Overflow::Hidden || self.style.overflow == Overflow::Scroll || self.style.overflow == Overflow::Auto {
                self.layout.block_extent = self.style.height.value();
            }
        }

        // Parent height can depend on child height, so `calculate_height` must be called after the
        // children are laid out.
        self.layout.content_box.height = if self.style.height.is_auto() {
            if self.is_block_root() {
                block_cursor.max(self.layout.block_extent) - self.layout.content_box.y
            } else {
                block_cursor - self.layout.content_box.y
            }
        } else {
            self.style.height.value()
        };
        // println!("====");
        // println!("self.layout.content_box.y: {}",self.layout.content_box.y);
        // println!("computed block_cursor: {}",block_cursor);
        // println!("computed content_box.height: {}",self.layout.content_box.height);
        // println!("====");

        // println!("before content_box.y: {}",self.layout.content_box.y);
        self.layout.padding_box = self.layout.content_box.extend_by(&self.layout.padding);
        self.layout.border_box = self.layout.padding_box.extend_by(&self.layout.border);
        self.layout.margin_box = self.layout.border_box.extend_by(&self.layout.effective_margin);
        // println!("====");
        // println!("computed self.layout.content_box.y:{}",self.layout.content_box.y);
        // println!("self.layout.margin.top:{}",self.layout.margin.top);
        // println!("self.layout.effective_margin.top:{}",self.layout.effective_margin.top);
        // println!("computed self.layout.border_box.y:{}",self.layout.border_box.y);
        // println!("====");

        self.layout.block_size = if self.is_in_flow() {
            // ==JUFIX== //
            if self.layout.border_box.height == 0.0 && !self.is_floated() {
            // if self.layout.content_box.height == 0.0 && !self.is_floated() {
                self.layout.effective_margin.top.max(self.layout.effective_margin.bottom)
            } else {
                self.layout.margin_box.height.max(0.0)
            }
        } else {
            0.0
        };
        // println!("computed block_size: {}",self.layout.block_size);


        // ==JUFIX:4==
        // NOTICE: this is NOT a good fix, still need to consider the margin collapsing
        if self.style.position == Positioned::Absolute {
            self.layout.block_extent = 0.0f32;
        }
        else {
            // ==JUFIX== QuickFix
            if self.style.position == Positioned::Relative {
                if let Some(rel_top) = self.style.top {
                    self.layout.block_extent = self.layout.block_extent.max(
                        // self.layout.margin_box.y + self.layout.margin_box.height
                        self.layout.border_box.y + self.layout.border_box.height
                    ) - rel_top;
                }
                else {
                    self.layout.block_extent = self.layout.block_extent.max(
                        // self.layout.margin_box.y + self.layout.margin_box.height
                        self.layout.border_box.y + self.layout.border_box.height
                    );
                }
            }
            else {
                self.layout.block_extent = self.layout.block_extent.max(
                    // self.layout.margin_box.y + self.layout.margin_box.height
                    self.layout.border_box.y + self.layout.border_box.height
                );
            }
            
        }
        // self.layout.block_extent = self.layout.block_extent.max(
        //     self.layout.margin_box.y + self.layout.margin_box.height
        // );

        self.layout.float_cursor = match self.style.float {
            Floated::Left => Lazy::new(self.layout.float_cursor.insert_left(&self.layout.margin_box)),
            Floated::Right => Lazy::new(self.layout.float_cursor.insert_right(&self.layout.margin_box)),
            Floated::None if self.class==LayoutClass::InlineBlock => Lazy::new(self.layout.float_cursor.insert_left_lh(&self.layout.margin_box,5.6f32)),
            Floated::None => self.layout.float_cursor.clone(),
        };

        // println!("computed float cursor left block end: {}",self.layout.float_cursor.left_block_end);
    }

    /// Calculate the width of a block-level non-replaced element in normal flow.
    ///
    /// http://www.w3.org/TR/CSS2/visudet.html#blockwidth
    ///
    /// Sets the horizontal margin/padding/border dimensions, and the `width`.
    fn calculate_block_width(&mut self) {

        // println!("====");
        // println!("self.style.width.is_auto: {}", self.style.width.is_auto());
        // println!("self.style.margin.left.is_auto: {}", self.style.margin.left.is_auto());
        // println!("self.style.margin.right.is_auto: {}", self.style.margin.right.is_auto());
        // println!("====");
        
        // Adjust used values to balance this difference, by increasing the total width by exactly
        // `underflow` pixels.
        self.layout.underflow = self.layout.containing_box.width - [
            self.style.margin.left.value(), self.style.margin.right.value(),
            self.style.border.left, self.style.border.right,
            self.style.padding.left, self.style.padding.right,
            self.style.width.value(),
        ].iter().sum::<f32>();

        self.layout.content_box.width = if self.style.width.is_auto() {
            if self.style.position == Positioned::Fixed {
                0.0
            }
            else {
                self.layout.underflow.max(0.0)
            }
        } else {
            self.style.width.value()
        };
        
        // Adjust used values to balance this difference, by increasing the total width by exactly
        // `underflow` pixels.
        // self.layout.underflow = self.layout.containing_box.width - [
        //     self.style.margin.left.value(), self.style.margin.right.value(),
        //     self.style.border.left, self.style.border.right,
        //     self.style.padding.left, self.style.padding.right,
        //     self.style.width.value(),
        // ].iter().sum::<f32>();

        // self.layout.content_box.width = if self.style.width.is_auto() {
        //     self.layout.underflow.max(0.0)
        // } else {
        //     self.style.width.value()
        // };

        self.layout.margin.left = if self.style.margin.left.is_auto() {
            if self.style.width.is_auto() || self.layout.underflow < 0.0 {
                0.0
            } else if self.style.margin.right.is_auto() {
                self.layout.underflow / 2.0
            } else {
                self.layout.underflow
            }
        } else {
            self.style.margin.left.value()
        };

        self.layout.margin.right = if self.style.width.is_auto() && self.layout.underflow < 0.0 {
            self.style.margin.right.value() + self.layout.underflow
        } else if self.style.margin.right.is_auto() {
            if self.style.width.is_auto() {
                0.0
            } else if self.style.margin.left.is_auto() {
                self.layout.underflow / 2.0
            } else {
                self.layout.underflow
            }
        } else if !self.style.margin.left.is_auto() || !self.style.width.is_auto() {
            self.style.margin.right.value() + self.layout.underflow
        } else {
            self.style.margin.right.value()
        };
    }

    /// Lay out a floating element and its descendants.
    fn layout_float(&mut self) {
        // println!("====");
        println!("call layout_float");
        self.layout.padding = self.style.padding;
        self.layout.border = self.style.border;

        if self.style.clear.left {
            self.layout.block_pos = self.layout.block_pos.max(
                self.layout.float_cursor.left_clearance()
            );
        }
        if self.style.clear.right {
            self.layout.block_pos = self.layout.block_pos.max(
                self.layout.float_cursor.right_clearance()
            );
        }

        self.layout.content_box.width = if self.style.width.is_auto() {
            // min(max(preferred_minimum_width, available_width), preferred_width)
            self.layout.containing_box.width
            - self.layout.padding.left
            - self.layout.padding.right
            - self.layout.border.left
            - self.layout.border.right
            - self.layout.effective_margin.left
            - self.layout.effective_margin.right
            // - self.layout.margin.left
            // - self.layout.margin.right
        } else {
            self.style.width.value()
        };
        self.layout.content_box.height = self.style.height.value();

        let mut available = self.layout.containing_box.clone();
        available.y = self.layout.block_pos;
        let outer_width =
            self.layout.content_box.width
            + self.layout.padding.left
            + self.layout.padding.right
            + self.layout.border.left
            + self.layout.border.right
            + self.layout.effective_margin.left
            + self.layout.effective_margin.right;
            // + self.layout.margin.left
            // + self.layout.margin.right;
        let (inline, block) = if self.is_floated_left() {
            self.layout.float_cursor.place_left(&available, outer_width)
        } else /* self.is_floated_right() */ {
            self.layout.float_cursor.place_right(&available, outer_width)
        };

        self.layout.content_box.x =
            inline
            + self.layout.padding.left
            + self.layout.border.left
            + self.layout.effective_margin.left;
            // + self.layout.margin.left;
        self.layout.content_box.y =
            block
            + self.layout.padding.top
            + self.layout.border.top
            + self.layout.effective_margin.top;
            // + self.layout.margin.top;

        // println!("available.x: {}",available.x);
        // println!("available.y: {}",available.y);
        // println!("available.width: {}",available.width);
        // println!("available.height: {}",available.height);
        // println!("float self.layout.content_box.width: {}",self.layout.content_box.width);
        // println!("outer_width: {}",outer_width);
        // println!("======");
        // println!("self.layout.containing_box.x: {}",self.layout.containing_box.x);
        // println!("======");
        // println!("float cursor left: {}",self.layout.float_cursor.left_block_end);
        // println!("available.width: {}",available.width);
        // println!("available.height: {}",available.height);
        // println!("outer_width: {}",outer_width);
        // println!("computed inline: {}",inline);
        // println!("computed block: {}",block);
        // println!("======");
        // println!("float left: {}",self.is_floated_left());
        // println!("float right: {}",self.is_floated_right());
        // println!("float self.layout.content_box.x: {}",self.layout.content_box.x);
        
        // println!("float cursor left: {}",self.layout.float_cursor.left_block_end);
        // println!("float cursor right: {}",self.layout.float_cursor.right_block_end);

        // Recursively lay out the children of this box.
        let inline_cursor = self.layout.content_box.x;
        let mut block_cursor = self.layout.content_box.y;
        let mut inner_float_cursor = FloatCursor::empty();
        for child in &mut self.children {
            // Give the child box the boundaries of its container.
            child.layout.containing_box = self.layout.content_box;
            child.layout.positioning_box = if self.style.position == Positioned::Static {
                self.layout.positioning_box
            } else {
                self.layout.content_box
            };
            child.layout.block_pos = block_cursor;
            child.layout.inline_pos = inline_cursor;
            // child.layout.float_cursor = FloatCursor::empty();
            child.layout.float_cursor = inner_float_cursor.clone();
            // Lay out the child box.
            child.layout();
            // Increment the cursor so each child is laid out below the previous one.
            block_cursor += child.layout.block_size;

            self.layout.block_extent = self.layout.block_extent.max(child.layout.block_extent);
            inner_float_cursor = child.layout.float_cursor.clone();
            // self.layout.float_cursor = child.layout.float_cursor.clone();
        }

        // Parent height can depend on child height, so `calculate_height` must be called after the
        // children are laid out.
        self.layout.content_box.height = if self.style.height.is_auto() {
            if self.is_block_root() {
                block_cursor.max(self.layout.block_extent) - self.layout.content_box.y
            } else {
                block_cursor - self.layout.content_box.y
            }
        } else {
            self.style.height.value()
        };
        // println!("float self.layout.content_box.height: {}",self.layout.content_box.height);

        self.layout.padding_box = self.layout.content_box.extend_by(&self.layout.padding);
        self.layout.border_box = self.layout.padding_box.extend_by(&self.layout.border);
        self.layout.margin_box = self.layout.border_box.extend_by(&self.layout.margin);
        // println!("computed self.layout.content_box.y: {}",self.layout.content_box.y);
        // println!("computed self.layout.border_box.y: {}",self.layout.border_box.y);

        // XXX: Use border box or margin box?
        self.layout.block_size = 0.0;
        self.layout.block_extent = self.layout.block_extent.max(
            if self.layout.border_box.height > 0.0 {
                self.layout.border_box.y + self.layout.border_box.height
            } else {
                0.0
            }
        );

        // match self.style.float {
        //     Floated::Left if self.layout.border_box.height>0.0 => println!("b1"),
        //     Floated::Right if self.layout.border_box.height>0.0 => println!("b2"),
        //     Floated::None => println!("b3"),
        //     _ => println!("b4"),
        // };
        // println!("margin box x:{}, y:{}, width:{}, height:{}",self.layout.margin_box.x, self.layout.margin_box.y, self.layout.margin_box.width, self.layout.margin_box.height);
        // println!("(bf) fl float cursor left: {}",self.layout.float_cursor.left_block_end);
        self.layout.float_cursor = match self.style.float {
            Floated::Left if self.layout.border_box.height>0.0 => Lazy::new(self.layout.float_cursor.insert_left(&self.layout.margin_box)),
            Floated::Right if self.layout.border_box.height>0.0 => Lazy::new(self.layout.float_cursor.insert_right(&self.layout.margin_box)),
            Floated::None => self.layout.float_cursor.clone(),
            _ => self.layout.float_cursor.clone(),
        };


        // println!("(af) fl float cursor left: {}",self.layout.float_cursor.left_block_end);
        // println!("(af) self.layout.border_box.width: {}",self.layout.border_box.width);
        // println!("======");

        // println!("final self.layout.content_box.x: {}",self.layout.content_box.x);
        self.layout.inline_size = self.layout.border_box.width;
        // println!("final self.layout.inline_size: {}",self.layout.inline_size);

        // println!("float cursor left: {}",self.layout.float_cursor.left_block_end);
        // println!("float cursor right: {}",self.layout.float_cursor.right_block_end);
        // println!("====");
    }

    fn render(&self, list: &mut DisplayList) {
        let block = self.layout.border_box;
        let frame = self.layout.border_box.frame_by(&self.layout.border);
        list.display_block(self.style.background_color, block);
        list.display_frame(self.style.border_color, frame);
        for child in self.children.iter().rev() {
            child.render(list);
        }
    }
}
