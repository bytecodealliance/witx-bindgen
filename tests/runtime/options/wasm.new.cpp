#include <assert.h>
#include <limits.h>
#include <math.h>
#include <options_cpp.h>

template <class T>
static bool equal(T const& a, T const& b) {
    return a==b;
}
static bool equal(wit::string const& a, std::string const& b) {
    return a.get_view() == std::string_view(b);
}
static bool equal(std::optional<wit::string> const& a, std::optional<std::string> const& b) {
    if (a.has_value() != b.has_value()) return false;
    if (a.has_value()) {
        return equal(a.value(), b.value());
    }
    return true;
}

void exports::options::TestImports() {
    using namespace test::options::test;

    OptionNoneParam(std::optional<std::string_view>());
    OptionSomeParam(std::optional<std::string_view>("foo"));
    assert(!OptionNoneResult());
    assert(equal(OptionSomeResult(), std::optional<std::string>("foo")));
    assert(equal(OptionRoundtrip(std::optional<std::string_view>("foo")), std::optional<std::string>("foo")));
    assert(equal(DoubleOptionRoundtrip(std::optional<std::optional<uint32_t>>(std::optional<uint32_t>(42))), std::optional<std::optional<uint32_t>>(std::optional<uint32_t>(42))));
    assert(equal(DoubleOptionRoundtrip(std::optional<std::optional<uint32_t>>(std::optional<uint32_t>())), std::optional<std::optional<uint32_t>>(std::optional<uint32_t>())));
    assert(equal(DoubleOptionRoundtrip(std::optional<std::optional<uint32_t>>()), std::optional<std::optional<uint32_t>>()));
}

void exports::test::options::test::OptionNoneParam(std::optional<std::string_view> a)
{
    assert(!a.has_value());
}

std::optional<wit::string> exports::test::options::test::OptionNoneResult() {
    return std::optional<wit::string>();
}

void exports::test::options::test::OptionSomeParam(std::optional<std::string_view> a) {
    assert(equal(a, std::optional<std::string_view>("foo")));
}

std::optional<wit::string> exports::test::options::test::OptionSomeResult() {
    return std::optional<wit::string>(wit::string::from_view("foo"));
}

std::optional<wit::string> exports::test::options::test::OptionRoundtrip(std::optional<std::string_view> a) {
    if (!a.has_value()) return std::optional<wit::string>();
    return std::optional<wit::string>(wit::string::from_view(*a));
}

std::optional<std::optional<uint32_t>> exports::test::options::test::DoubleOptionRoundtrip(std::optional<std::optional<uint32_t>> a) {
    return a;
}
