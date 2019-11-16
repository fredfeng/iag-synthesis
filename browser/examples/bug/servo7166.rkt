;; From file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/servo7166.html

(define-stylesheet doc-1
  ((id first)
   [position absolute]
   [width (px 100)]
   [height (px 100)]
   [background-color red])
  ((id second)
   [position absolute]
   [top (px 50)]
   [left (px 50)]
   [width (px 100)]
   [height (px 100)]
   [background-color green])
  ((id third)
   [position fixed]))

(define-fonts doc-1
  [16 "serif" 400 normal 12 4 0 0 19.2])

(define-layout (doc-1 :matched true :w 1280 :h 663 :fs 16 :scrollw 0)
 ([VIEW :w 1280]
  ([BLOCK :x 0 :y 0 :w 1280 :h 8 :elt 0]
   ([BLOCK :x 8 :y 8 :w 1264 :h 0 :elt 3]
    ([BLOCK :x 8 :y 8 :w 100 :h 100 :elt 4]
     ([BLOCK :x 58 :y 58 :w 100 :h 100 :elt 5])
     ([BLOCK :x 8 :y 8 :w 0 :h 0 :elt 6]))))))

(define-document doc-1
  ([html :num 0]
   ([head :num 1]
    ([link :num 2]))
   ([body :num 3]
    ([span :num 4 :id first]
     ([span :num 5 :id second]) " "
     ([span :num 6 :id third])) " ")))

(define-problem doc-1
  :title ""
  :url "file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/servo7166.html"
  :sheets firefox doc-1
  :fonts doc-1
  :documents doc-1
  :layouts doc-1
  :features css:position empty-text float:0)

