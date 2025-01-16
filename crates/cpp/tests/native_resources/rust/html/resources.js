
export class R {
    constructor(value) {
        this.value = value;
    }
    add(b) {
        this.value += b;
    }
}

export function borrows(obj) {
    console.log('borrows', obj.value);
}

export function consume(obj) {
    console.log('consume', obj.value);
}

export function create() {
    return new R(1);
}
