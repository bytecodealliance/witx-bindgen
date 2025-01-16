#include <assert.h>
#include <results_cpp.h>

template <class T>
bool equal(T const&a, T const&b) {
    return a==b;
}

std::expected<float, wit::string> exports::test::results::test::StringError(float a) {
    return ::test::results::test::StringError(a);
}

std::expected<float, ::test::results::test::E> exports::test::results::test::EnumError(float a) {
    auto result = ::test::results::test::EnumError(a);
    if (result.has_value()) { return result.value(); }
    return std::unexpected(result.error());
    // if (result.error()==::test::results::test::E::kA) { return std::unexpected(::test::results::test::E::kA); }
    // if (result.error()==::test::results::test::E::kB) { return std::unexpected(::test::results::test::E::kB); }
    // if (result.error()==::test::results::test::E::kC) { return std::unexpected(::test::results::test::E::kC); }
}

std::expected<float, ::test::results::test::E2> exports::test::results::test::RecordError(float a) {
    auto result = ::test::results::test::RecordError(a);
    if (result.has_value()) { return result.value(); }
    return std::unexpected(::test::results::test::E2{ result.error().line, result.error().column });
}

std::expected<float, ::test::results::test::E3> exports::test::results::test::VariantError(float a) {
    auto result = ::test::results::test::VariantError(a);
    if (result.has_value()) { return result.value(); }
    return std::unexpected(result.error());

    // match test_imports::variant_error(a) {
    //     Ok(b) => Ok(b),
    //     Err(test_imports::E3::E1(test_imports::E::A)) => {
    //         Err(test_exports::E3::E1(test_exports::E::A))
    //     }
    //     Err(test_imports::E3::E1(test_imports::E::B)) => {
    //         Err(test_exports::E3::E1(test_exports::E::B))
    //     }
    //     Err(test_imports::E3::E1(test_imports::E::C)) => {
    //         Err(test_exports::E3::E1(test_exports::E::C))
    //     }
    //     Err(test_imports::E3::E2(test_imports::E2 { line, column })) => {
    //         Err(test_exports::E3::E2(test_exports::E2 { line, column }))
    //     }
    // }
}

std::expected<uint32_t, wit::Void> exports::test::results::test::EmptyError(uint32_t a) {
    return ::test::results::test::EmptyError(a);
}

std::expected<std::expected<void, wit::string>, wit::string> exports::test::results::test::DoubleError(uint32_t a) {
    return ::test::results::test::DoubleError(a);
}
