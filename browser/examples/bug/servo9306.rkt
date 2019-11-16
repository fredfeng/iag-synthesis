;; From file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/servo9306.html

(define-stylesheet doc-1
  ((tag body)
   [margin-top (px 0)]
   [margin-right (px 0)]
   [margin-bottom (px 0)]
   [margin-left (px 0)])
  ((class green)
   [background-color lime]
   [height (px 40)]
   [width (px 400)]
   [top (px 40)]
   [left (px 727.5)]
   [margin-left (px -300)]
   [position absolute]
   [padding-top (px 0)]
   [padding-bottom (px 0)]
   [padding-left (px 100)]
   [padding-right (px 100)])
  ((class red)
   [width (px 8)]
   [height (px 19.2)]
   [background-color red]
   [position absolute]
   [right (px 0)]))

(define-fonts doc-1
  [16 "serif" 400 normal 12 4 0 0 19.2])

(define-layout (doc-1 :matched true :w 1280 :h 663 :fs 16 :scrollw 0)
 ([VIEW :w 1280]
  ([BLOCK :x 0 :y 0 :w 1280 :h 0 :elt 0]
   ([BLOCK :x 0 :y 0 :w 1280 :h 0 :elt 3]
    ([BLOCK :x 427.5 :y 40 :w 600 :h 40 :elt 4]
     ([BLOCK :x 1019.5 :y 40 :w 8 :h 19.2 :elt 5]))))))

(define-document doc-1
  ([html :num 0]
   ([head :num 1]
    ([link :num 2]))
   ([body :num 3]
    ([div :num 4 :class (green)] " "
     ([div :num 5 :class (red)])) " ")))

(define-problem doc-1
  :title ""
  :url "file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/servo9306.html"
  :sheets firefox doc-1
  :fonts doc-1
  :documents doc-1
  :layouts doc-1
  :features css:position empty-text float:0)

