#include <assert.h>
#include <records_cpp.h>

template <class T>
bool equal(T const&a, T const&b) {
    return a==b;
}

void exports::records::TestImports() {
    using namespace ::test::records::test;

    assert(equal(MultipleResults(), std::tuple<uint8_t, uint16_t>(4, 5)));

    assert(equal(SwapTuple(std::tuple<uint8_t, uint32_t>(1, 2)), std::tuple<uint32_t, uint8_t>(2, 1)));
    assert(equal(RoundtripFlags1(::test::records::test::F1::kA), ::test::records::test::F1::kA));
    assert(equal(RoundtripFlags1(::test::records::test::F1::k_None), ::test::records::test::F1::k_None));
    assert(equal(RoundtripFlags1(::test::records::test::F1::kB), ::test::records::test::F1::kB));
    assert(equal(RoundtripFlags1(::test::records::test::F1::kA | ::test::records::test::F1::kB), ::test::records::test::F1::kA | ::test::records::test::F1::kB));

    assert(equal(RoundtripFlags2(::test::records::test::F2::kC), ::test::records::test::F2::kC));
    assert(equal(RoundtripFlags2(::test::records::test::F2::k_None), ::test::records::test::F2::k_None));
    assert(equal(RoundtripFlags2(::test::records::test::F2::kD), ::test::records::test::F2::kD));
    assert(equal(RoundtripFlags2(::test::records::test::F2::kC | ::test::records::test::F2::kE), ::test::records::test::F2::kC | ::test::records::test::F2::kE));

    assert(equal(
        RoundtripFlags3(::test::records::test::Flag8::kB0, ::test::records::test::Flag16::kB1, ::test::records::test::Flag32::kB2),
        std::tuple<::test::records::test::Flag8, ::test::records::test::Flag16, ::test::records::test::Flag32>(::test::records::test::Flag8::kB0, ::test::records::test::Flag16::kB1, ::test::records::test::Flag32::kB2)
    ));

    {
        auto r = RoundtripRecord1(::test::records::test::R1 {
            8,
            ::test::records::test::F1::k_None,
        });
        assert(equal(r.a, (uint8_t)8));
        assert(equal(r.b, ::test::records::test::F1::k_None));
    }

    auto r = RoundtripRecord1(::test::records::test::R1 {
        0,
        ::test::records::test::F1::kA | ::test::records::test::F1::kB,
    });
    assert(equal(r.a, (uint8_t)0));
    assert(equal(r.b, ::test::records::test::F1::kA | ::test::records::test::F1::kB));

    assert(equal(Tuple1(std::tuple<uint8_t>(1)), std::tuple<uint8_t>(1)));
}

std::tuple<uint8_t, uint16_t> exports::test::records::test::MultipleResults() {
    return std::tuple<uint8_t, uint16_t>(100, 200);
}

std::tuple<uint32_t, uint8_t> exports::test::records::test::SwapTuple(std::tuple<uint8_t, uint32_t> a) {
    return std::tuple<uint32_t, uint8_t>(std::get<1>(a), std::get<0>(a));
}

test::records::test::F1 exports::test::records::test::RoundtripFlags1(::test::records::test::F1 a) {
    return a;
}

test::records::test::F2 exports::test::records::test::RoundtripFlags2(::test::records::test::F2 a) {
    return a;
}

std::tuple<test::records::test::Flag8, test::records::test::Flag16, test::records::test::Flag32> exports::test::records::test::RoundtripFlags3(::test::records::test::Flag8 a, ::test::records::test::Flag16 b, ::test::records::test::Flag32 c) {
    return std::tuple<::test::records::test::Flag8, ::test::records::test::Flag16, ::test::records::test::Flag32>(a, b, c);
}

test::records::test::R1 exports::test::records::test::RoundtripRecord1(::test::records::test::R1 a) {
    return a;
}

std::tuple<uint8_t> exports::test::records::test::Tuple1(std::tuple<uint8_t> a) {
    return std::tuple<uint8_t>(std::get<0>(a));
}
