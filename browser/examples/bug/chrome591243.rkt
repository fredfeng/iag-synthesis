;; From file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/chrome591243.html

(define-stylesheet doc-1
  ((tag body)
   [width (px 100)]
   [border-top-width (px 3)]
   [border-right-width (px 3)]
   [border-bottom-width (px 3)]
   [border-left-width (px 3)]
   [border-top-style solid]
   [border-right-style solid]
   [border-bottom-style solid]
   [border-left-style solid]
   #;[border-top-color red]
   #;[border-right-color red]
   #;[border-bottom-color red]
   #;[border-left-color red])
  ((id div-first-child)
   [float right]
   [height (px 30)]
   [width (px 30)]
   [background-color blue]
   #;[background-position-x (% 0)]
   #;[background-position-y (% 0)]
   #;[background-repeat repeat]
   #;[background-attachment scroll]
   #;[background-image none]
   #;[background-size auto]
   #;[background-origin padding-box]
   #;[background-clip border-box])
  ((tag em)
   [display block]
   [clear both]
   [width (px 16)]
   [height (px 16)])
  ((id em-before)
   [display block]
   [width (px 16)]
   [height (px 0)]
   [overflow-x hidden]
   [overflow-y hidden]
   [clear both])
  ((id em-after)
   [margin-top (px -30)])
  ((id div-child)
   [display inline-block]
   [width (px 50)]
   [height (px 30)]
   [background-color yellow]
   #;[background-position-x (% 0)]
   #;[background-position-y (% 0)]
   #;[background-repeat repeat]
   #;[background-attachment scroll]
   #;[background-image none]
   #;[background-size auto]
   #;[background-origin padding-box]
   #;[background-clip border-box]))

(define-fonts doc-1
  [16 "serif" 400 normal 12 4 0 0 19.2]
  [16 "serif" 400 italic 11 4 0.5 0.5 19.2])

(define-layout (doc-1 :matched true :w 1280 :h 663 :fs 16 :scrollw 0)
 ([VIEW :w 1280]
  ([BLOCK :x 0 :y 0 :w 1280 :h 27 :elt 0]
   ([BLOCK :x 8 :y 8 :w 106 :h 6 :elt 3]
    ([BLOCK :x 11 :y 11 :w 16 :h 0 :elt 4])
    ([BLOCK :x 11 :y 11 :w 16 :h 16 :elt 5])
    ([BLOCK :x 11 :y -3 :w 100 :h 0 :elt 6]
     ([BLOCK :x 81 :y -3 :w 30 :h 30 :elt 7])
     ([ANON]
      ([LINE]
       ([INLINE :elt 8]))))))))

(define-document doc-1
  ([html :num 0]
   ([head :num 1]
    ([link :num 2]))
   ([body :num 3]
    ([span :num 4 :id em-before])
    ([em :num 5])
    ([div :num 6 :id em-after]
     ([span :num 7 :id div-first-child :class (div-child)]) " "
     ([span :num 8 :class (div-child)])) " ")))

(define-problem doc-1
  :title ""
  :url "file:///Users/joseph/Desktop/UCSB/19fall/layout/iag-synthesis/browser/examples/bug/chrome591243.html"
  :sheets firefox doc-1
  :fonts doc-1
  :documents doc-1
  :layouts doc-1
  :features css:float css:clear css:overflow-x css:overflow-y empty-text float:1)

