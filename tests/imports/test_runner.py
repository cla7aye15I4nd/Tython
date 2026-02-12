from . import module_a
from . import module_a as mod_a
from . import module_b
from . import module_c
from . import module_d
from . import module_e
from . import module_f
from . import module_g
from . import module_h

from .module_a import func_a, compute_a
from .nested.deep.deeper import bottom
from .nested.deep.deeper import complex_imports
from .nested.deep.deeper.deepest import deepest_imports
from .nested.deep.deeper.bottom import bottom_func, bottom_compute
from .nested.deep import mid_level
from .nested.deep import cross_level_imports
from .nested import top_level
from . import class_provider_a
from . import class_provider_b
from .class_provider_a import Vec2 as ImportedVec2
from .class_provider_nested import Outer, Deep


def test_module_a() -> None:
    print(module_a.func_a())
    print('CHECK test_runner lhs expr:', 'module_a.func_a()')
    print('CHECK test_runner rhs:', 1)
    assert module_a.func_a() == 1
    print(module_a.compute_a(10))
    print('CHECK test_runner lhs expr:', 'module_a.compute_a(10)')
    print('CHECK test_runner rhs:', 11)
    assert module_a.compute_a(10) == 11


def test_module_b() -> None:
    print(module_b.func_b())
    print('CHECK test_runner lhs expr:', 'module_b.func_b()')
    print('CHECK test_runner rhs:', 3)
    assert module_b.func_b() == 3
    print(module_b.compute_b(5))
    print('CHECK test_runner lhs expr:', 'module_b.compute_b(5)')
    print('CHECK test_runner rhs:', 12)
    assert module_b.compute_b(5) == 12


def test_module_c() -> None:
    print(module_c.func_c())
    print('CHECK test_runner lhs expr:', 'module_c.func_c()')
    print('CHECK test_runner rhs:', 7)
    assert module_c.func_c() == 7
    print(module_c.compute_c(5))
    print('CHECK test_runner lhs expr:', 'module_c.compute_c(5)')
    print('CHECK test_runner rhs:', 18)
    assert module_c.compute_c(5) == 18


def test_module_d() -> None:
    print(module_d.func_d())
    print('CHECK test_runner lhs expr:', 'module_d.func_d()')
    print('CHECK test_runner rhs:', 11)
    assert module_d.func_d() == 11
    print(module_d.compute_d(5))
    print('CHECK test_runner lhs expr:', 'module_d.compute_d(5)')
    print('CHECK test_runner rhs:', 21)
    assert module_d.compute_d(5) == 21


def test_nested_bottom() -> None:
    print(bottom.bottom_func())
    print('CHECK test_runner lhs expr:', 'bottom.bottom_func()')
    print('CHECK test_runner rhs:', 100)
    assert bottom.bottom_func() == 100
    print(bottom.bottom_compute(10))
    print('CHECK test_runner lhs expr:', 'bottom.bottom_compute(10)')
    print('CHECK test_runner rhs:', 110)
    assert bottom.bottom_compute(10) == 110


def test_nested_mid() -> None:
    print(mid_level.mid_func())
    print('CHECK test_runner lhs expr:', 'mid_level.mid_func()')
    print('CHECK test_runner rhs:', 110)
    assert mid_level.mid_func() == 110
    print(mid_level.mid_compute(10))
    print('CHECK test_runner lhs expr:', 'mid_level.mid_compute(10)')
    print('CHECK test_runner rhs:', 120)
    assert mid_level.mid_compute(10) == 120


def test_nested_top() -> None:
    print(top_level.top_func())
    print('CHECK test_runner lhs expr:', 'top_level.top_func()')
    print('CHECK test_runner rhs:', 111)
    assert top_level.top_func() == 111
    print(top_level.top_compute(10))
    print('CHECK test_runner lhs expr:', 'top_level.top_compute(10)')
    print('CHECK test_runner rhs:', 121)
    assert top_level.top_compute(10) == 121


def test_module_e() -> None:
    print(module_e.module_e_func())
    print('CHECK test_runner lhs expr:', 'module_e.module_e_func()')
    print('CHECK test_runner rhs:', 326)
    assert module_e.module_e_func() == 326
    print(module_e.module_e_compute(5))
    print('CHECK test_runner lhs expr:', 'module_e.module_e_compute(5)')
    print('CHECK test_runner rhs:', 105)
    assert module_e.module_e_compute(5) == 105


def test_module_f() -> None:
    print(module_f.module_f_func())
    print('CHECK test_runner lhs expr:', 'module_f.module_f_func()')
    print('CHECK test_runner rhs:', 327)
    assert module_f.module_f_func() == 327
    print(module_f.module_f_compute(10))
    print('CHECK test_runner lhs expr:', 'module_f.module_f_compute(10)')
    print('CHECK test_runner rhs:', 230)
    assert module_f.module_f_compute(10) == 230


def test_module_g() -> None:
    result = module_g.module_g_func()
    print(result)
    print('CHECK test_runner lhs:', result)
    print('CHECK test_runner rhs:', 1940)
    assert result == 1940
    print(module_g.module_g_use_e())
    print('CHECK test_runner lhs expr:', 'module_g.module_g_use_e()')
    print('CHECK test_runner rhs:', 326)
    assert module_g.module_g_use_e() == 326
    print(module_g.module_g_compute(10))
    print('CHECK test_runner lhs expr:', 'module_g.module_g_compute(10)')
    print('CHECK test_runner rhs:', 517)
    assert module_g.module_g_compute(10) == 517


def test_complex_imports_same_dir() -> None:
    print(complex_imports.test_same_dir())
    print('CHECK test_runner lhs expr:', 'complex_imports.test_same_dir()')
    print('CHECK test_runner rhs:', 100)
    assert complex_imports.test_same_dir() == 100


def test_complex_imports_parent() -> None:
    print(complex_imports.test_parent_import())
    print('CHECK test_runner lhs expr:', 'complex_imports.test_parent_import()')
    print('CHECK test_runner rhs:', 110)
    assert complex_imports.test_parent_import() == 110


def test_complex_imports_grandparent() -> None:
    print(complex_imports.test_grandparent_import())
    print('CHECK test_runner lhs expr:', 'complex_imports.test_grandparent_import()')
    print('CHECK test_runner rhs:', 111)
    assert complex_imports.test_grandparent_import() == 111


def test_complex_imports_great_grandparent() -> None:
    print(complex_imports.test_great_grandparent_import())
    print('CHECK test_runner lhs expr:', 'complex_imports.test_great_grandparent_import()')
    print('CHECK test_runner rhs:', 4)
    assert complex_imports.test_great_grandparent_import() == 4


def test_complex_imports_specific() -> None:
    print(complex_imports.test_specific_imports())
    print('CHECK test_runner lhs expr:', 'complex_imports.test_specific_imports()')
    print('CHECK test_runner rhs:', 328)
    assert complex_imports.test_specific_imports() == 328


def test_complex_imports_chain() -> None:
    print(complex_imports.complex_chain())
    print('CHECK test_runner lhs expr:', 'complex_imports.complex_chain()')
    print('CHECK test_runner rhs:', 321)
    assert complex_imports.complex_chain() == 321


def test_complex_imports_compute() -> None:
    print(complex_imports.complex_compute(5))
    print('CHECK test_runner lhs expr:', 'complex_imports.complex_compute(5)')
    print('CHECK test_runner rhs:', 118)
    assert complex_imports.complex_compute(5) == 118


def test_cross_level() -> None:
    print(cross_level_imports.cross_level_func())
    print('CHECK test_runner lhs expr:', 'cross_level_imports.cross_level_func()')
    print('CHECK test_runner rhs:', 322)
    assert cross_level_imports.cross_level_func() == 322


def test_cross_level_complex() -> None:
    print(cross_level_imports.test_complex_imports_module())
    print('CHECK test_runner lhs expr:', 'cross_level_imports.test_complex_imports_module()')
    print('CHECK test_runner rhs:', 100)
    assert cross_level_imports.test_complex_imports_module() == 100


def test_cross_level_sibling() -> None:
    print(cross_level_imports.test_sibling_interaction())
    print('CHECK test_runner lhs expr:', 'cross_level_imports.test_sibling_interaction()')
    print('CHECK test_runner rhs:', 1322)
    assert cross_level_imports.test_sibling_interaction() == 1322


def test_cross_level_renamed() -> None:
    print(cross_level_imports.test_renamed_import())
    print('CHECK test_runner lhs expr:', 'cross_level_imports.test_renamed_import()')
    print('CHECK test_runner rhs:', 111)
    assert cross_level_imports.test_renamed_import() == 111


def test_cross_level_chain() -> None:
    print(cross_level_imports.test_deep_chain())
    print('CHECK test_runner lhs expr:', 'cross_level_imports.test_deep_chain()')
    print('CHECK test_runner rhs:', 321)
    assert cross_level_imports.test_deep_chain() == 321


def test_cross_level_compute() -> None:
    print(cross_level_imports.compute_cross(5))
    print('CHECK test_runner lhs expr:', 'cross_level_imports.compute_cross(5)')
    print('CHECK test_runner rhs:', 24)
    assert cross_level_imports.compute_cross(5) == 24


def test_module_h() -> None:
    print(module_h.module_h_deep_access())
    print('CHECK test_runner lhs expr:', 'module_h.module_h_deep_access()')
    print('CHECK test_runner rhs:', 530)
    assert module_h.module_h_deep_access() == 530
    print(module_h.module_h_nested_chain())
    print('CHECK test_runner lhs expr:', 'module_h.module_h_nested_chain()')
    print('CHECK test_runner rhs:', 229)
    assert module_h.module_h_nested_chain() == 229
    print(module_h.module_h_complex_chain())
    print('CHECK test_runner lhs expr:', 'module_h.module_h_complex_chain()')
    print('CHECK test_runner rhs:', 333)
    assert module_h.module_h_complex_chain() == 333
    print(module_h.module_h_cross_level())
    print('CHECK test_runner lhs expr:', 'module_h.module_h_cross_level()')
    print('CHECK test_runner rhs:', 1652)
    assert module_h.module_h_cross_level() == 1652
    print(module_h.module_h_compute(5))
    print('CHECK test_runner lhs expr:', 'module_h.module_h_compute(5)')
    print('CHECK test_runner rhs:', 24)
    assert module_h.module_h_compute(5) == 24


def test_deepest_imports_same_dir() -> None:
    print(deepest_imports.test_same_dir())
    print('CHECK test_runner lhs expr:', 'deepest_imports.test_same_dir()')
    print('CHECK test_runner rhs:', 1000)
    assert deepest_imports.test_same_dir() == 1000


def test_deepest_imports_parent() -> None:
    print(deepest_imports.test_parent_import())
    print('CHECK test_runner lhs expr:', 'deepest_imports.test_parent_import()')
    print('CHECK test_runner rhs:', 100)
    assert deepest_imports.test_parent_import() == 100


def test_deepest_imports_grandparent() -> None:
    print(deepest_imports.test_grandparent_import())
    print('CHECK test_runner lhs expr:', 'deepest_imports.test_grandparent_import()')
    print('CHECK test_runner rhs:', 110)
    assert deepest_imports.test_grandparent_import() == 110


def test_deepest_imports_great_grandparent() -> None:
    print(deepest_imports.test_great_grandparent_import())
    print('CHECK test_runner lhs expr:', 'deepest_imports.test_great_grandparent_import()')
    print('CHECK test_runner rhs:', 111)
    assert deepest_imports.test_great_grandparent_import() == 111


def test_deepest_imports_five_level() -> None:
    print(deepest_imports.test_five_level_import())
    print('CHECK test_runner lhs expr:', 'deepest_imports.test_five_level_import()')
    print('CHECK test_runner rhs:', 4)
    assert deepest_imports.test_five_level_import() == 4


def test_deepest_imports_all_levels() -> None:
    print(deepest_imports.test_all_levels())
    print('CHECK test_runner lhs expr:', 'deepest_imports.test_all_levels()')
    print('CHECK test_runner rhs:', 1325)
    assert deepest_imports.test_all_levels() == 1325


def test_deepest_imports_compute() -> None:
    print(deepest_imports.deepest_compute(5))
    print('CHECK test_runner lhs expr:', 'deepest_imports.deepest_compute(5)')
    print('CHECK test_runner rhs:', 1012)
    assert deepest_imports.deepest_compute(5) == 1012


def test_module_h_deepest() -> None:
    print(module_h.module_h_deepest())
    print('CHECK test_runner lhs expr:', 'module_h.module_h_deepest()')
    print('CHECK test_runner rhs:', 1325)
    assert module_h.module_h_deepest() == 1325


def test_import_aliases() -> None:
    print(mod_a.func_a())
    print('CHECK test_runner lhs expr:', 'mod_a.func_a()')
    print('CHECK test_runner rhs:', 1)
    assert mod_a.func_a() == 1


def test_from_import_specific() -> None:
    print(func_a())
    print('CHECK test_runner lhs expr:', 'func_a()')
    print('CHECK test_runner rhs:', 1)
    assert func_a() == 1
    print(compute_a(20))
    print('CHECK test_runner lhs expr:', 'compute_a(20)')
    print('CHECK test_runner rhs:', 21)
    assert compute_a(20) == 21


def test_from_import_nested() -> None:
    print(bottom_func())
    print('CHECK test_runner lhs expr:', 'bottom_func()')
    print('CHECK test_runner rhs:', 100)
    assert bottom_func() == 100
    print(bottom_compute(5))
    print('CHECK test_runner lhs expr:', 'bottom_compute(5)')
    print('CHECK test_runner rhs:', 105)
    assert bottom_compute(5) == 105


def test_cross_module_class_construct() -> None:
    p: class_provider_a.Vec2 = class_provider_a.Vec2(3, 4)
    print(p.x)
    print('CHECK test_runner lhs:', p.x)
    print('CHECK test_runner rhs:', 3)
    assert p.x == 3
    print(p.y)
    print('CHECK test_runner lhs:', p.y)
    print('CHECK test_runner rhs:', 4)
    assert p.y == 4
    print(p.sum())
    print('CHECK test_runner lhs expr:', 'p.sum()')
    print('CHECK test_runner rhs:', 7)
    assert p.sum() == 7


def test_cross_module_class_method() -> None:
    a: class_provider_a.Vec2 = class_provider_a.Vec2(1, 2)
    b: class_provider_a.Vec2 = class_provider_a.Vec2(3, 4)
    d: int = class_provider_a.dot_vec2(a, b)
    print(d)
    print('CHECK test_runner lhs:', d)
    print('CHECK test_runner rhs:', 11)
    assert d == 11


def test_cross_module_class_field_mutation() -> None:
    p: class_provider_a.Vec2 = class_provider_a.Vec2(5, 6)
    p.x = 50
    print(p.x)
    print('CHECK test_runner lhs:', p.x)
    print('CHECK test_runner rhs:', 50)
    assert p.x == 50
    print(p.sum())
    print('CHECK test_runner lhs expr:', 'p.sum()')
    print('CHECK test_runner rhs:', 56)
    assert p.sum() == 56


def test_cross_module_factory_function() -> None:
    p: class_provider_a.Vec2 = class_provider_a.make_vec2(10, 20)
    print(p.x)
    print('CHECK test_runner lhs:', p.x)
    print('CHECK test_runner rhs:', 10)
    assert p.x == 10
    print(p.y)
    print('CHECK test_runner lhs:', p.y)
    print('CHECK test_runner rhs:', 20)
    assert p.y == 20


def test_cross_module_same_name_no_collision() -> None:
    a: class_provider_a.Vec2 = class_provider_a.Vec2(1, 2)
    b: class_provider_b.Vec2 = class_provider_b.Vec2(1, 2, 3)
    print(a.sum())
    print('CHECK test_runner lhs expr:', 'a.sum()')
    print('CHECK test_runner rhs:', 3)
    assert a.sum() == 3
    print(b.sum())
    print('CHECK test_runner lhs expr:', 'b.sum()')
    print('CHECK test_runner rhs:', 6)
    assert b.sum() == 6


def test_from_import_class() -> None:
    p: ImportedVec2 = ImportedVec2(9, 4)
    print(p.sum())
    print('CHECK test_runner lhs expr:', 'p.sum()')
    print('CHECK test_runner rhs:', 13)
    assert p.sum() == 13


def test_from_import_nested_class() -> None:
    outer: Outer = Outer(21)
    inner: Outer.Inner = Outer.Inner(8)
    print(outer.get_base())
    print('CHECK test_runner lhs expr:', 'outer.get_base()')
    print('CHECK test_runner rhs:', 21)
    assert outer.get_base() == 21
    print(inner.get())
    print('CHECK test_runner lhs expr:', 'inner.get()')
    print('CHECK test_runner rhs:', 8)
    assert inner.get() == 8


def test_from_import_deep_nested_class() -> None:
    leaf: Deep.Mid.Leaf = Deep.Mid.Leaf(7)
    print(leaf.triple())
    print('CHECK test_runner lhs expr:', 'leaf.triple()')
    print('CHECK test_runner rhs:', 21)
    assert leaf.triple() == 21


def run_all_tests() -> None:
    test_module_a()
    test_module_b()
    test_module_c()
    test_module_d()
    test_nested_bottom()
    test_nested_mid()
    test_nested_top()
    test_module_e()
    test_module_f()
    test_module_g()
    test_complex_imports_same_dir()
    test_complex_imports_parent()
    test_complex_imports_grandparent()
    test_complex_imports_great_grandparent()
    test_complex_imports_specific()
    test_complex_imports_chain()
    test_complex_imports_compute()
    test_cross_level()
    test_cross_level_complex()
    test_cross_level_sibling()
    test_cross_level_renamed()
    test_cross_level_chain()
    test_cross_level_compute()
    test_module_h()
    test_deepest_imports_same_dir()
    test_deepest_imports_parent()
    test_deepest_imports_grandparent()
    test_deepest_imports_great_grandparent()
    test_deepest_imports_five_level()
    test_deepest_imports_all_levels()
    test_deepest_imports_compute()
    test_module_h_deepest()
    test_import_aliases()
    test_from_import_specific()
    test_from_import_nested()
    test_cross_module_class_construct()
    test_cross_module_class_method()
    test_cross_module_class_field_mutation()
    test_cross_module_factory_function()
    test_cross_module_same_name_no_collision()
    test_from_import_class()
    test_from_import_nested_class()
    test_from_import_deep_nested_class()
