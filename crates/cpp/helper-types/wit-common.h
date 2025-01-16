#pragma once

#include <assert.h>
#include <map>
#include <optional>
#include <stddef.h> // size_t
#include <stdint.h>
#if __cplusplus > 202001L
#include <span>
#else
#include <vector>
#endif

namespace wit {
#if __cplusplus > 202001L
using std::span;
#else
/// Minimal span (vector view) implementation for older C++ environments
template <class T> class span {
  T const *address;
  size_t length;

public:
  T const *data() const { return address; }
  size_t size() const { return length; }

  typedef T const *const_iterator;

  const_iterator begin() const { return address; }
  const_iterator end() const { return address + length; }
  bool empty() const { return !length; }
  T const &operator[](size_t index) const { return address[index]; }
  span(T *a, size_t l) : address(a), length(l) {}
  // create from any compatible vector (borrows data!)
  template <class U>
  span(std::vector<U> const &vec) : address(vec.data()), length(vec.size()) {}
};
#endif

/// @brief Helper class to map between IDs and resources
/// @tparam R Type of the Resource
template <class R> class ResourceTable {
  static std::map<int32_t, R> resources;

public:
  static R *lookup_resource(int32_t id) {
    auto result = resources.find(id);
    return result == resources.end() ? nullptr : &result->second;
  }
  static int32_t store_resource(R &&value) {
    auto last = resources.rbegin();
    int32_t id = last == resources.rend() ? 0 : last->first + 1;
    resources.insert(std::pair<int32_t, R>(id, std::move(value)));
    return id;
  }
  static std::optional<R> remove_resource(int32_t id) {
    auto iter = resources.find(id);
    std::optional<R> result;
    if (iter != resources.end()) {
      result = std::move(iter->second);
      resources.erase(iter);
    }
    return std::move(result);
  }
};

/// @brief Replaces void in the error position of a result
struct Void {};
} // namespace wit
