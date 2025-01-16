#include <assert.h>
#include <many_arguments_cpp.h>

template <class T>
bool equal(T const&a, T const&b) {
    return a==b;
}

void exports::many_arguments::ManyArguments(
    uint64_t a1,
    uint64_t a2,
    uint64_t a3,
    uint64_t a4,
    uint64_t a5,
    uint64_t a6,
    uint64_t a7,
    uint64_t a8,
    uint64_t a9,
    uint64_t a10,
    uint64_t a11,
    uint64_t a12,
    uint64_t a13,
    uint64_t a14,
    uint64_t a15,
    uint64_t a16
) {
    assert(equal(a1, (uint64_t)1));
    assert(equal(a2, (uint64_t)2));
    assert(equal(a3, (uint64_t)3));
    assert(equal(a4, (uint64_t)4));
    assert(equal(a5, (uint64_t)5));
    assert(equal(a6, (uint64_t)6));
    assert(equal(a7, (uint64_t)7));
    assert(equal(a8, (uint64_t)8));
    assert(equal(a9, (uint64_t)9));
    assert(equal(a10, (uint64_t)10));
    assert(equal(a11, (uint64_t)11));
    assert(equal(a12, (uint64_t)12));
    assert(equal(a13, (uint64_t)13));
    assert(equal(a14, (uint64_t)14));
    assert(equal(a15, (uint64_t)15));
    assert(equal(a16, (uint64_t)16));
    ::test::many_arguments::ManyArguments(
        a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16
    );
}
