;; From file:///Users/yufeng/research/other/iag-synthesis/browser/examples/sanity/floats-004.html

(define-stylesheet doc-1
  ((id div1)
   [height (px 96)]
   [width (px 96)])
  ((id div2)
   [height (px 96)]
   [width (px 96)])
  ((id div1)
   [background-color orange]
   #;[background-position-x (% 0)]
   #;[background-position-y (% 0)]
   #;[background-repeat repeat]
   #;[background-attachment scroll]
   #;[background-image none]
   #;[background-size auto]
   #;[background-origin padding-box]
   #;[background-clip border-box]
   [float right])
  ((id div2)
   [background-color blue]
   #;[background-position-x (% 0)]
   #;[background-position-y (% 0)]
   #;[background-repeat repeat]
   #;[background-attachment scroll]
   #;[background-image none]
   #;[background-size auto]
   #;[background-origin padding-box]
   #;[background-clip border-box]
   [float right]))

(define-fonts doc-1
  [16 "serif" 400 normal 12 4 0 0 19.2])

(define-layout (doc-1 :matched true :w 1280 :h 737 :fs 16 :scrollw 0)
 ([VIEW :w 1280]
  ([BLOCK :x 0 :y 0 :w 1280 :h 104 :elt 0]
   ([BLOCK :x 8 :y 8 :w 1264 :h 0 :elt 3]
    ([BLOCK :x 8 :y 8 :w 1264 :h 0 :elt 4]
     ([BLOCK :x 1176 :y 8 :w 96 :h 96 :elt 5])
     ([BLOCK :x 1080 :y 8 :w 96 :h 96 :elt 6]))))))

(define-document doc-1
  ([html :num 0]
   ([head :num 1]
    ([link :num 2]))
   ([body :num 3]
    ([div :num 4]
     ([div :num 5 :id div1]) " "
     ([div :num 6 :id div2])) " ")))

(define-problem doc-1
  :title ""
  :url "file:///Users/yufeng/research/other/iag-synthesis/browser/examples/sanity/floats-004.html"
  :sheets firefox doc-1
  :fonts doc-1
  :documents doc-1
  :layouts doc-1
  :features css:float empty-text float:1)

