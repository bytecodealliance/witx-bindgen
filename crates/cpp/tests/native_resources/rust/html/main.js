import {resources} from './component.js'

var r = resources.create();
r.add(12);
resources.borrows(r);
resources.consume(r);
let s = new resources.R(42);
s = null;
