monitor swdp_scan
attach 1
break DefaultHandler
break UserHardFault
break rust_begin_unwind
set mem inaccessible-by-default off
load
compare-sections
