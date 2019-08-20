#lang rosette

(require "../utility.rkt"
         "../grammar/syntax.rkt")

(provide generate-program)

; -----------------------
; Standard program header
; -----------------------

(define directives
  (list '(hash-bang allow (unused_parens))
        '(hash-bang allow (unused_variables))
        '(hash-bang allow (dead_code))
        '(blank)))

(define imports
  (list '(use crate dom DocumentNode)
        '(use crate style (StyledTree StyledNode Style DisplayType Floated Positioned))
        '(use crate paint DisplayList)
        '(use crate utility (Pixels Edge Rect FloatCursor))
        '(use crate utility (MarginAccumulator FloatCursor))
        '(use crate lazy Lazy)
        '(use itertools Itertools)))

(define struct-Layout
  (list '(blank)
        '(hash derive (Clone Default PartialEq Debug))
        '(struct Layout ()
           (record (: container (gen Rect (Pixels)))
                   (: content_box (gen Rect (Pixels)))
                   (: padding_box (gen Rect (Pixels)))
                   (: border_box (gen Rect (Pixels)))
                   (: margin_box (gen Rect (Pixels)))
                   (: flow_height Pixels)
                   (: margin_clear bool)
                   (: margin_weird bool)
                   (: margin_above MarginAccumulator)
                   (: margin_below MarginAccumulator)
                   (: padding (gen Edge (Pixels)))
                   (: border (gen Edge (Pixels)))
                   (: margin (gen Edge (Pixels)))
                   (: float_cursor (gen Lazy (FloatCursor)))
                   (: underflow Pixels)))))

(define struct-LayoutBox
  (list '(blank)
        '(struct LayoutBox ((life a))
           (record (: element (gen Option ((ref a DocumentNode))))
                   (: class LayoutClass)
                   (: layout Layout)
                   (: style (ref a Style))
                   (: children (gen Vec ((gen LayoutNode ((life a))))))))))

(define impl-LayoutBox-new
  (list '(blank)
        '(impl ((life a)) LayoutBox
               (fn new () ((: box_type BoxType) (: style (ref a Style))) Self
                   (do (return (struct LayoutBox
                                 ((: element None)
                                  (: container (call (:: Rect default) ()))
                                  (: content_box (call (:: Rect default) ()))
                                  (: padding_box (call (:: Rect default) ()))
                                  (: border_box (call (:: Rect default) ()))
                                  (: margin_box (call (:: Rect default) ()))
                                  (: computedHeight 0.0)
                                  (: margin_acc (call (:: MarginAccumulator default) ()))
                                  (: floatLstIn (call (:: FloatList empty) ()))
                                  (: floatLstOut (call (:: FloatList empty) ()))
                                  (: padding (call (:: Edge default) ()))
                                  (: border (call (:: Edge default) ()))
                                  (: margin (call (:: Edge default) ()))
                                  (: underflow 0.0)
                                  (: style style)
                                  (: anonymous #t)
                                  (: class box_type)
                                  (: children (call (:: Vec new) ()))))))))))

(define impl-Display-for-LayoutBox
  (list '(blank)
        '(impl ((life a)) (for (:: fmt Display) LayoutBox)
               (fn fmt () ((: self (ref Self)) (: f (ref (mut (:: fmt Formatter))))) (:: fmt Result)
                          (do (if (== (select self class) (:: BoxType None))
                                  (do (? (call write! (f "#|")))))
                              (? (call write! (f "(")))
                              (match (select self element)
                                (=> (constructor None (unit))
                                    (do (? (call write! (f "[ANON]")))))
                                (=> (constructor Some (tuple id))
                                    (do (let kind
                                             (match (select self class)
                                                (=> (constructor (:: BoxType None) (unit)) "NONE")
                                                (=> (constructor (:: BoxType Block) (unit)) "BLOCK")
                                                (=> (constructor (:: BoxType Float) (unit)) "BLOCK")
                                                (=> (constructor (:: BoxType Inline) (unit)) "INLINE")))
                                        (let x (select (select self content_box) x))
                                        (let y (select (select self content_box) y))
                                        (let w (select (select self content_box) width))
                                        (let h (select (select self content_box) height))
                                        (? (call write! (f "[{} :x {} :y {} :w {} :h {} :elt {}]" kind x y w h id))))))
                              (for child (call (select (select self children) iter) ())
                                (do (? (call write! (f " {}" child)))))
                              (? (call write! (f ")")))
                              (if (== (select self class) (:: BoxType None))
                                  (do (? (call write! (f "|#")))))
                              (return (call (:: Result Ok) ((unit)))))))))

(define fn-layout_tree
  (list '(blank)
        '(pub (fn layout_tree ((life a)) ((: node (ref a (gen StyledNode ((life a)))))
                                          (: width usize)
                                          (: height usize))
                  (gen LayoutBox ((life a)))
                  (do (let-mut root_box (call build_layout_tree (node)))
                      (:= (select (select root_box container) width) (as width Pixels))
                    (call (select root_box layout) ())
                    (call println! ("(define-layout (doc-2 :matched true :w {} :h {} :fs 16 :scrollw 0) ([VIEW :w {}] {}))"
                                    width height width root_box))
                    (return root_box))))))

(define fn-build_layout_tree
  (list '(blank)
        '(fn build_layout_tree ((life a)) ((: style_node (ref a (gen StyledNode ((life a))))))
             (gen LayoutBox ((life a)))
             (do (let box_type (match (select (select style_node specified) display)
                                 (=> (constructor (:: Display Inline) (unit)) (:: BoxType Inline))
                                 (=> (constructor (:: Display Block) (unit)) (:: BoxType Block))
                                 (=> (constructor (:: Display Float) (unit)) (:: BoxType Float))
                                 (=> (constructor (:: Display None) (unit)) (:: BoxType None))))
                 (let style (ref (select style_node specified)))
                 (let-mut root (call (:: LayoutBox new) (box_type style)))
                 (:= (select root element) (call Some ((select (select style_node node) number))))
                 (:= (select root anonymous) #f)
                 (let children (call (select (call (select (select style_node children) iter) ()) map) (build_layout_tree)))
                 (for (tuple box_type children) (call (select (ref children) group_by) ((lambda (child) (select child class))))
                      (do (if (!= (select root class) box_type)
                              (do (let-mut wrapper (call (:: LayoutBox new) ((select root class) style)))
                                  (call (select (select wrapper children) extend) (children))
                                  (call (select (select root children) push) (wrapper)))
                              (do (call (select (select root children) extend) (children))))))
                 (return root)))))

(define fn-display_list
  (list '(blank)
        '(pub (fn display_list ((life a)) ((: layout_root (ref (gen LayoutBox ((life a)))))) DisplayList
              (do (let-mut list (call (:: Vec new) ()))
                  (call (select layout_root render) ((ref (mut list))))
                  (return list))))))

(define impl-LayoutBox-render
  (list '(blank)
        '(impl ((life a)) LayoutBox
               (fn render () ((: self (ref Self)) (: list (ref (mut DisplayList)))) (unit)
                   (do (call (select list push)
                             ((struct (:: DisplayCommand SolidColor)
                                ((: color (select (select self style) background_color))
                                 (: x (select (select self border_box) x))
                                 (: y (select (select self border_box) y))
                                 (: width (select (select self border_box) width))
                                 (: height (select (select self border_box) height))))))
                     (call (select list push)
                           ((struct (:: DisplayCommand SolidColor)
                              ((: color (select (select self style) border_color))
                               (: x (select (select self border_box) x))
                               (: y (select (select self border_box) y))
                               (: width (select (select self border) left))
                               (: height (select (select self border_box) height))))))
                     (call (select list push)
                           ((struct (:: DisplayCommand SolidColor)
                              ((: color (select (select self style) border_color))
                               (: x (- (+ (select (select self border_box) x)
                                          (select (select self border_box) width))
                                       (select (select self border) right)))
                               (: y (select (select self border_box) y))
                               (: width (select (select self border) right))
                               (: height (select (select self border_box) height))))))
                     (call (select list push)
                           ((struct (:: DisplayCommand SolidColor)
                              ((: color (select (select self style) border_color))
                               (: x (select (select self border_box) x))
                               (: y (select (select self border_box) y))
                               (: width (select (select self border_box) width))
                               (: height (select (select self border) top))))))
                     (call (select list push)
                           ((struct (:: DisplayCommand SolidColor)
                              ((: color (select (select self style) border_color))
                               (: x (select (select self border_box) x))
                               (: y (- (+ (select (select self border_box) y)
                                          (select (select self border_box) height))
                                       (select (select self border) bottom)))
                               (: width (select (select self border_box) width))
                               (: height (select (select self border) bottom))))))


                     (for child (call (select (call (select (select self children) iter) ()) rev) ())
                          (do (call (select child render) (list)))))))))

(define header
  (append directives
          imports
          struct-Layout
          struct-LayoutBox
          impl-LayoutBox-new
          impl-Display-for-LayoutBox
          fn-layout_tree
          fn-build_layout_tree
          fn-display_list
          impl-LayoutBox-render))

; ---------------------------------
; Generation of tree data structure
; ---------------------------------

(define/match (generate-child-field child)
  [((ag:child/one name (ag:interface sort _ _)))
   `(: ,name (gen Box (,sort)))]
  [((ag:child/seq name (ag:interface sort _ _)))
   `(: ,name (gen Vec (,sort)))])

(define/match (generate-label-field label)
  [((ag:label name type))
   `(: ,name ,type)])

(define (generate-class-field interface)
  (define sort (symbol-append (ag:interface-name interface) 'Class))
  `(: class ,sort))

(define (generate-class-variant class)
  (define name (ag:class-name class))
  (define fields (map generate-child-field (ag:class-children* class)))
  `(variant ,name (record . ,fields)))

(define (generate-interface-enumeration interface)
  (define sort (symbol-append (ag:interface-name interface) 'Class))
  (define classes (ag:interface-classes interface))

  `(enum ,sort () . ,(map generate-class-variant classes)))

(define (generate-interface-structure interface)
  (define sort (ag:interface-name interface))
  (define fields
    (cons (generate-class-field interface)
          (map generate-label-field (ag:interface-labels interface))))

  `(struct ,sort () (record . ,fields)))

(define (generate-class-enumeration interface)
  (define sort (ag:interface-name interface))
  (define type-name (symbol-append sort 'Class))

  (define variants
    (for/list ([class (ag:interface-classes interface)])
      (define tag (ag:class-name class))
      `(variant ,tag (unit))))

  (list '(blank)
        '(hash derive (Clone Copy PartialEq Eq Debug))
        `(enum ,type-name . ,variants)))

(define (generate-structure G)
  (define interfaces (ag:grammar-interfaces G))
  (define classes (ag:grammar-classes G))

  (append (map generate-interface-structure interfaces)
          (map generate-interface-enumeration interfaces)))

; ---------------------------------
; Generation of tree traversal code
; ---------------------------------

(define (generate-term class term)
  (define/match (recur term)
    [((ag:const v)) v]
    [((or (ag:field (cons 'self 'intrinsic_height))
          (ag:accum (cons 'self 'intrinsic_height))))
     `(select (select self content_box) height)]
    [((ag:field (cons 'self field)))
     `(select self ,field)]
    [((ag:field (cons child field)))
     #:when (ag:child/seq? (ag:class-ref*/child class child))
     (define child-i (symbol-append child '_i))
     `(select ,child-i ,field)]
    [((ag:field (cons child field)))
     `(select ,child ,field)]
    [((ag:accum (cons object field)))
     `(select ,object ,field)]
    [((ag:index (cons child field) default))
     (define endpoint (if (ag:index/first? term) 'first 'last))
     (define first-child `(call (select ,child ,endpoint) ()))
     `(call (select ,first-child map_or_else)
            ((lambda () ,(recur default))
             (lambda (node) (select node ,field))))]
    [((ag:ite condition consequent alternate))
     `(if ,(recur condition)
          ,(recur consequent)
          ,(recur alternate))]
    [((ag:expr operator operands))
     `(,operator . ,(map recur operands))]
    [((ag:call function (list)))
     `(call ,function ())]
    [((ag:call function (cons receiver arguments)))
     `(call (select ,(recur receiver) ,function)
            ,(map recur arguments))])
  (recur term))

(define (generate-command function class command #:iterated? [iterated? #f])
  (define recur (curry generate-command function class))
  (match command
    [(ag:iter child commands)
     (define reversed? (ag:iter-rev? command))
     (define iterator `(call (select (select self ,child) iter_mut) ()))
     (define cursor (symbol-append child '_i))
     (define initial (append-map (recur #:iterated? #f) (flatten commands)))
     (define action (append-map (recur #:iterated? #t) (flatten commands)))
     (append initial
             (list `(for ,cursor ,(if reversed? `(call (select ,iterator rev) ()) iterator)
                      (do . ,action))))]
    [(ag:eval attr)
     (define rule (ag:class-ref*/rule class attr))
     (define iterator (ag:rule-iteration rule))
     (define term
       (match (ag:rule-formula rule)
         [(ag:fold init next) (if iterated? next init)]
         [term term]))
     (define target (generate-term class (ag:field attr)))
     (if (implies (ag:rule-iteration rule) (or (ag:rule-folds? rule) iterated?))
         (list `(:= ,target ,(generate-term class term)))
         null)]
    [(ag:recur child)
     (define child-i (symbol-append child '_i))
     (if (implies (ag:child/seq? (ag:class-ref*/child class child)) iterated?)
         (let ([receiver (if iterated? child-i child)])
           (list `(call (select ,receiver ,function) ())))
         null)]
    [(ag:skip)
     (list `(skip))]))

(define (generate-visitor name visitor)
  (define class (ag:visitor-class visitor))
  (define interface (ag:class-interface class))
  (define commands (ag:visitor-commands visitor))

  (define sort (symbol-append (ag:interface-name interface) 'Class))
  (define kind (ag:class-name class))
  (define variant `(:: ,sort ,kind))
  (define fields (map ag:child-name (ag:class-children* class)))
  (define pattern `(constructor ,variant (record . ,fields)))
  (define body (append-map (curry generate-command name class) (flatten commands)))

  `(=> (constructor (:: LayoutClass ,kind) (unit)) (do . ,body)))

(define (generate-traversal G traversal)
  (define name (ag:traversal-name traversal))

  (for/list ([interface (ag:grammar-interfaces G)]
             #:when (eq? (ag:interface-name interface) 'LayoutBox))
    (define sort (ag:interface-name interface))
    (define cases
      (map (curry generate-visitor name)
           (ag:traversal-ref/interface traversal interface)))

    `(impl ((life a)) ,sort
           (fn ,name () ((: self (ref (mut Self)))) (unit)
               (do (match (select self class)
                     (=> (constructor (:: LayoutClass None) (unit)) (skip))
                     .
                     ,cases))))))

(define (generate-program G S)
  (append ;header
          ;(add-between (generate-structure G) `(blank))
          (list `(blank))
          (add-between (generate-traversal G S) `(blank))))
