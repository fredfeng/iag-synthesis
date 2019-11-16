;; From file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/servo16737.html

(define-stylesheet doc-1
  ((class outer)
   [padding-top (px 10)]
   [padding-right (px 10)]
   [padding-bottom (px 10)]
   [padding-left (px 10)]
   [background-color darkblue]
   #;[background-position-x (% 0)]
   #;[background-position-y (% 0)]
   #;[background-repeat repeat]
   #;[background-attachment scroll]
   #;[background-image none]
   #;[background-size auto]
   #;[background-origin padding-box]
   #;[background-clip border-box]
   [color white]
   [height (px 16)])
  ((class inner)
   [width (px 16)]
   [height (px 16)]
   [background-color red]
   #;[background-position-x (% 0)]
   #;[background-position-y (% 0)]
   #;[background-repeat repeat]
   #;[background-attachment scroll]
   #;[background-image none]
   #;[background-size auto]
   #;[background-origin padding-box]
   #;[background-clip border-box])
  ((id lhs)
   [display inline-block])
  ((id rhs)
   [float right]))

(define-fonts doc-1
  [16 "serif" 400 normal 12 4 0 0 19.2])

(define-layout (doc-1 :matched true :w 1280 :h 663 :fs 16 :scrollw 0)
 ([VIEW :w 1280]
  ([BLOCK :x 0 :y 0 :w 1280 :h 52 :elt 0]
   ([BLOCK :x 8 :y 8 :w 1264 :h 36 :elt 4]
    ([BLOCK :x 8 :y 8 :w 1264 :h 36 :elt 5]
     ([LINE]
      ([INLINE :x 18 :y 18 :w 16 :h 16 :elt 6])
      ([TEXT :x 34 :y 22 :w 0 :h 16 :text " "])
      ([BLOCK :x 1246 :y 18 :w 16 :h 16 :elt 7])))))))

(define-document doc-1
  ([html :num 0]
   ([head :num 1]
    ([title :num 2])
    ([link :num 3]))
   ([body :num 4]
    ([div :num 5 :class (outer)]
     ([div :num 6 :id lhs :class (inner)]) " "
     ([div :num 7 :id rhs :class (inner)])) " ")))

(define-problem doc-1
  :title "Float block is positioned incorrectly within block with text"
  :url "file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/servo16737.html"
  :sheets firefox doc-1
  :fonts doc-1
  :documents doc-1
  :layouts doc-1
  :features css:float display:inline-block float:1)

