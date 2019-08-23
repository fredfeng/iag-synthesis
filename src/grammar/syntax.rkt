#lang rosette

; Abstract syntax for language of attribute grammars

(require "../utility.rkt")

(provide (prefix-out ag:
                     (except-out (all-defined-out)
                                 grammar
                                 list->grammar
                                 grammar-link!
                                 make-child
                                 make-label
                                 make-rule
                                 make-fold
                                 make-scan
                                 make-field
                                 make-iter
                                 set-class-interface!
                                 set-class-traits!
                                 set-class-counters!
                                 set-child-interface!
                                 set-rule-iteration!
                                 set-visitor-class!)))

; TODO: In the future, it would be nice to isolate the rest of the codebase from
; the particulars of the program representation as much as possible.

; attribute grammar
(struct grammar (interfaces classes traits traversals))

; interface definition
(struct interface (name labels [classes #:mutable #:auto]) #:auto-value #f)

; trait definition
(struct trait (name body) #:transparent)

; class definition
(struct class (name [interface #:mutable] [traits #:mutable] body [counters #:mutable #:auto]) #:auto-value #f)

; class/trait body
(struct body (children labels rules))

; child declaration
(struct child (name [interface #:mutable]) #:constructor-name make-child)
(struct child/one child ())
(struct child/seq child ())

; label declaration
(struct label (name type) #:constructor-name make-label)
(struct label/in label ())
(struct label/out label ())
(struct label/var label ())

; rule definition
(struct rule (attribute formula [iteration #:mutable]) #:transparent #:constructor-name make-rule)
(struct rule/scalar rule () #:transparent)
(struct rule/vector rule () #:transparent)
(define rule-object (compose car rule-attribute))
(define rule-field (compose cdr rule-attribute))

; formulae
(struct fold (init next) #:transparent #:constructor-name make-fold)
(struct fold/left fold () #:transparent)
(struct fold/right fold () #:transparent)
(struct scan (init next) #:transparent #:constructor-name make-scan)
(struct scan/left scan () #:transparent)
(struct scan/right scan () #:transparent)

; terms
(struct const (value) #:transparent)
(struct field (attribute) #:transparent #:constructor-name make-field)
(struct field/get field () #:transparent)
(struct field/cur field () #:transparent)
(struct field/acc field () #:transparent)
(struct field/sup field () #:transparent)
(struct field/peek field (default) #:transparent)
(struct field/pred field/peek () #:transparent)
(struct field/succ field/peek () #:transparent)
(struct field/first field (default) #:transparent)
(struct field/last field (default) #:transparent)
(struct length (child) #:transparent)
(struct expr (operator operands) #:transparent)
(struct call (function arguments) #:transparent)
(struct invoke (receiver method arguments) #:transparent)
(struct branch (if then else) #:transparent)
(define term? (disjoin const? field? length? expr? call? invoke? branch?))

; traversal definition
(struct traversal (name visitors) #:transparent)

; visitor case
(struct visitor ([class #:mutable] commands) #:transparent)

(struct iter (child commands) #:transparent #:constructor-name make-iter)
(struct iter/left iter () #:transparent)
(struct iter/right iter () #:transparent)
(struct recur (child) #:transparent)
(struct eval (attribute) #:transparent)
(struct skip () #:transparent)
(struct hole () #:transparent)
(define command? (disjoin iter? recur? eval? skip? hole?))

; traversal composition (into schedules)
(struct sequential (left right) #:transparent)
(struct parallel (left right) #:transparent)

; traversal invocation
(struct traverse (order) #:transparent)

; ---------------------
; Utilities for grammar
; ---------------------

(define (list->grammar entities)
  (let*-values ([(interfaces other) (partition interface? entities)]
                [(classes other) (partition class? other)]
                [(traits other) (partition trait? other)]
                [(traversals other) (partition traversal? other)])

    (unless (null? other)
      (raise-argument-error 'make-grammar
                            "(or/c interface? class? trait? traversal?)"
                            (first other)))
    (grammar interfaces classes traits traversals)))

; "Hot-link" the AST to avoid repetitive look-ups
(define (grammar-link! G)
  (for ([interface (grammar-interfaces G)])
    (define name (interface-name interface))
    (define classes (filter-by class-interface (grammar-classes G) name))
    (set-interface-classes! interface classes))

  (for* ([trait (grammar-traits G)]
         [child (trait-children trait)])
    (define interface (grammar-ref/interface G (child-interface child)))
    (set-child-interface! child interface))

  (for ([class (grammar-classes G)])
    (for ([child (class-children class)])
      (define interface (grammar-ref/interface G (child-interface child)))
      (set-child-interface! child interface))

    (define interface (grammar-ref/interface G (class-interface class)))
    (define traits (map (curry grammar-ref/trait G) (class-traits class)))
    (set-class-interface! class interface)
    (set-class-traits! class traits)

    (set-class-counters! class
                         (for/list ([rule (class-rules* class)]
                                    #:when (rule-iterative? rule))
                           (rule-attribute rule)))

    (for ([rule (class-rules* class)])
      (set-rule-iteration! rule (rule-iterates class rule))))

  (for* ([trav (grammar-traversals G)]
         [visitor (traversal-visitors trav)])
    (define class (grammar-ref/class G (visitor-class visitor)))
    (set-visitor-class! visitor class)))

(define (make-grammar entities)
  (let ([G (list->grammar entities)])
    (grammar-link! G)
    G))

(define (grammar-ref/interface G name)
  (or (find-by interface-name (grammar-interfaces G) name)
      (raise-user-error 'grammar-ref "Undefined interface '~a'" name)))

(define (grammar-ref/class G name)
  (or (find-by class-name (grammar-classes G) name)
      (raise-user-error 'grammar-ref "Undefined class '~a'" name)))

(define (grammar-ref/trait G name)
  (or (find-by trait-name (grammar-traits G) name)
      (raise-user-error 'grammar-ref "Undefined trait '~a'" name)))

(define (grammar-ref/traversal G name)
  (or (find-by traversal-name (grammar-traversals G) name)
      (raise-user-error 'grammar-ref "Undefined traversal '~a'" name)))

; -----------------------
; Utilities for interface
; -----------------------

(define (interface-ref/label interface name)
  (find-by label-name (interface-labels interface) name))

; -------------------
; Utilities for trait
; -------------------

(define trait-children (compose body-children trait-body))
(define trait-labels (compose body-labels trait-body))
(define trait-rules (compose body-rules trait-body))

(define (trait-ref/child trait name)
  (body-ref/child (trait-body trait) name))

(define (trait-ref/label trait name)
  (body-ref/label (trait-body trait) name))

(define (trait-ref/rule trait attribute)
  (body-ref/rule (trait-body trait) attribute))

; -------------------
; Utilities for class
; -------------------

(define class-interface-name (compose interface-name class-interface))

(define class-children (compose body-children class-body))
(define class-labels (compose body-labels class-body))
(define class-rules (compose body-rules class-body))

(define (class-ref/child class name)
  (body-ref/child (class-body class) name))

(define (class-ref/label class name)
  (body-ref/label (class-body class) name))

(define (class-ref/rule class attribute)
  (body-ref/rule (class-body class) attribute))

(define (class-children* class)
  (union* (class-children class)
          (map trait-children (class-traits class))
          #:key child-name))

(define (class-labels* class)
  (union* (interface-labels (class-interface class))
          (class-labels class)
          (map trait-labels (class-traits class))
          #:key label-name))

(define (class-rules* class)
  (union* (class-rules class)
          (map trait-rules (class-traits class))
          #:key rule-attribute))

(define (class-accumulators* class)
  (map rule-attribute (filter rule-iterative? (class-rules* class))))

(define (class-ref*/child class name #:partial? [partial? #f])
  (or (class-ref/child class name)
      (ormap (λ (trait) (trait-ref/child trait name))
             (class-traits class))
      (if partial?
          #f
          (raise-user-error 'class-ref*
                            "Undefined child '~a' in class '~a'"
                            name (class-name class)))))

(define (class-ref*/label class name #:partial? [partial? #f])
  (or (interface-ref/label (class-interface class) name)
      (class-ref/label class name)
      (ormap (λ (trait) (trait-ref/label trait name))
             (class-traits class))
      (if partial?
          #f
          (raise-user-error 'class-ref*
                            "Undefined field '~a' in class '~a'"
                            name (class-name class)))))

(define (class-ref*/rule class attribute #:partial? [partial? #f])
  (or (class-ref/rule class attribute)
      (ormap (λ (trait) (trait-ref/rule trait attribute))
             (class-traits class))
      (if partial?
          #f
          (raise-user-error 'class-ref*
                            "Undefined rule for attribute '~a' in class '~a'"
                            (attribute->string attribute) (class-name class)))))

(define (class-child-seq? class name)
  (child/seq? (class-ref*/child class name)))

(define (class-child-one? class name)
  (child/one? (class-ref*/child class name)))

(define (class-interface-ref/label class name)
  (interface-ref/label (class-interface class) name))

; ------------------------------
; Utilities for class/trait body
; ------------------------------

(define/match (body-union body1 body2)
  [((body children1 labels1 rules1)
    (body children2 labels2 rules2))
   (body (append children1 children2)
         (append labels1 labels2)
         (append rules1 rules2))])

(define (body-ref/child body name)
  (find-by child-name (body-children body) name))

(define (body-ref/label body name)
  (find-by label-name (body-labels body) name))

(define (body-ref/rule body attribute)
  (find-by rule-attribute (body-rules body) attribute))

; -------------------
; Utilities for child
; -------------------

(define child-interface-name (compose interface-name child-interface))

; -----------------------
; Utilities for traversal
; -----------------------

(define (traversal-ref/visitor traversal class)
  (find-by visitor-class (traversal-visitors traversal)
           class #:same? eq?))

(define (traversal-ref/interface traversal interface)
  (filter-by visitor-interface (traversal-visitors traversal)
             interface #:same? eq?))

; --------------------
; Utlities for visitor
; --------------------

(define visitor-class-name (compose class-name visitor-class))
(define visitor-interface (compose class-interface visitor-class))
(define visitor-interface-name (compose interface-name visitor-interface))

; -------------
; Odds and ends
; -------------

(define (rule-iterative? rule)
  (match (rule-formula rule)
    [(fold _ _) #t] ; either `fold/left` or `fold/right`
    [(scan _ _) #t] ; either `scan/left` or `scan/right`
    [_ #f]))

(define (rule-step rule)
  (match (rule-formula rule)
    [(fold _ next) next]
    [(scan _ next) next]
    [term term]))

(define (rule-init rule)
  (match (rule-formula rule)
    [(fold init _) init] ; either `fold/left` or `fold/right`
    [(scan init _) init] ; either `scan/left` or `scan/right`
    [_ #f]))

(define (rule-next rule)
  (match (rule-formula rule)
    [(fold _ next) next] ; either `fold/left` or `fold/right`
    [(scan _ next) next] ; either `scan/left` or `scan/right`
    [_ #f]))

(define (rule-order rule)
  (match (rule-formula rule)
    [(fold/left _ _) 'left]
    [(fold/right _ _) 'right]
    [(scan/left _ _) 'left]
    [(scan/right _ _) 'right]
    [_ #f]))

(define (rule-iterates class rule)
  (match rule
    [(rule/vector (cons child _) _ _) child]
    [(rule/scalar _ (fold _ next) _) (term-iterates class next)]
    [_ #f]))

(define (term-iterates class term)
  (match term
    [(const _)
     #f]
    [(field/get _)
     #f]
    [(field/cur (cons child _))
     child]
    [(field/acc (cons object _))
     (if (child/seq? (class-ref*/child class object #:partial? #t))
         object
         ; NOTE: Really implies same iteration as the accumulator's rule, which
         ; is independent of the accumulating attribute's node for a fold rule.
         #f)]
    [(field/sup _)
     #f]
    [(field/peek (cons child _) _)
     child]
    [(field/first _ default)
     (term-iterates class default)]
    [(field/last _ default)
     (term-iterates class default)]
    [(length _)
     #f]
    [(expr _ operands)
     (ormap (curry term-iterates class) operands)]
    [(call _ arguments)
     (ormap (curry term-iterates class) arguments)]
    [(invoke receiver _ arguments)
     (ormap (curry term-iterates class) (cons receiver arguments))]
    [(branch if then else)
     (ormap (curry term-iterates class) (list if then else))]))

(define fold-rev? fold/right?)
(define scan-rev? scan/right?)
(define iter-rev? iter/right?)

(define (attribute->string attr)
  (format "~a.~a" (car attr) (cdr attr)))

(define/match (eval->rule class command)
  [(_ (eval attr)) (class-ref*/rule class attr)]
  [(_ _) #f])
