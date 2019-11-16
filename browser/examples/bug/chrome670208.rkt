;; From file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/chrome670208.html

(define-stylesheet doc-1
  ((class box)
   [width (px 220)]
   [height (px 220)]
   [overflow-x auto]
   [overflow-y auto]
   [border-top-width (px 1)]
   [border-right-width (px 1)]
   [border-bottom-width (px 1)]
   [border-left-width (px 1)]
   [border-top-style solid]
   [border-right-style solid]
   [border-bottom-style solid]
   [border-left-style solid]
   #;[border-top-color black]
   #;[border-right-color black]
   #;[border-bottom-color black]
   #;[border-left-color black])
  ((class item)
   [width (px 55)]
   [height (px 55)]
   [float right]
   [margin-left (px 10)]
   [margin-bottom (px 10)]
   [background-color red]
   [border-top-width (px 1)]
   [border-right-width (px 1)]
   [border-bottom-width (px 1)]
   [border-left-width (px 1)]
   [border-top-style solid]
   [border-right-style solid]
   [border-bottom-style solid]
   [border-left-style solid]
   #;[border-top-color transparent]
   #;[border-right-color transparent]
   #;[border-bottom-color transparent]
   #;[border-left-color transparent]))

(define-fonts doc-1
  [16 "serif" 400 normal 12 4 0 0 19.2])

(define-layout (doc-1 :matched true :w 1280 :h 663 :fs 16 :scrollw 0)
 ([VIEW :w 1280]
  ([BLOCK :x 0 :y 0 :w 1280 :h 238 :elt 0]
   ([BLOCK :x 8 :y 8 :w 1264 :h 222 :elt 4]
    ([BLOCK :x 8 :y 8 :w 222 :h 222 :elt 5]
     ([BLOCK :x 172 :y 9 :w 57 :h 57 :elt 6])
     ([BLOCK :x 105 :y 9 :w 57 :h 57 :elt 7])
     ([BLOCK :x 38 :y 9 :w 57 :h 57 :elt 8])
     ([BLOCK :x 172 :y 76 :w 57 :h 57 :elt 9])
     ([BLOCK :x 105 :y 76 :w 57 :h 57 :elt 10])
     ([BLOCK :x 38 :y 76 :w 57 :h 57 :elt 11])
     ([BLOCK :x 172 :y 143 :w 57 :h 57 :elt 12])
     ([BLOCK :x 105 :y 143 :w 57 :h 57 :elt 13])
     ([BLOCK :x 38 :y 143 :w 57 :h 57 :elt 14])
     ([BLOCK :x 172 :y 210 :w 57 :h 57 :elt 15])
     ([BLOCK :x 105 :y 210 :w 57 :h 57 :elt 16])
     ([BLOCK :x 38 :y 210 :w 57 :h 57 :elt 17])
     ([BLOCK :x 172 :y 277 :w 57 :h 57 :elt 18])
     ([BLOCK :x 105 :y 277 :w 57 :h 57 :elt 19])
     ([BLOCK :x 38 :y 277 :w 57 :h 57 :elt 20]))))))

(define-document doc-1
  ([html :num 0]
   ([head :num 1]
    ([link :num 2])
    ([title :num 3]))
   ([body :num 4]
    ([div :num 5 :class (box)]
     ([div :num 6 :class (item)]) " "
     ([div :num 7 :class (item)]) " "
     ([div :num 8 :class (item)]) " "
     ([div :num 9 :class (item)]) " "
     ([div :num 10 :class (item)]) " "
     ([div :num 11 :class (item)]) " "
     ([div :num 12 :class (item)]) " "
     ([div :num 13 :class (item)]) " "
     ([div :num 14 :class (item)]) " "
     ([div :num 15 :class (item)]) " "
     ([div :num 16 :class (item)]) " "
     ([div :num 17 :class (item)]) " "
     ([div :num 18 :class (item)]) " "
     ([div :num 19 :class (item)]) " "
     ([div :num 20 :class (item)])) " ")))

(define-problem doc-1
  :title "JS Bin"
  :url "file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/chrome670208.html"
  :sheets firefox doc-1
  :fonts doc-1
  :documents doc-1
  :layouts doc-1
  :features css:overflow-x overflow:auto css:overflow-y css:float empty-text float:2)

