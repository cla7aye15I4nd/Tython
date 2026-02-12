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
    assert module_a.func_a() == 1
    print(module_a.compute_a(10))
    assert module_a.compute_a(10) == 11


def test_module_b() -> None:
    print(module_b.func_b())
    assert module_b.func_b() == 3
    print(module_b.compute_b(5))
    assert module_b.compute_b(5) == 12


def test_module_c() -> None:
    print(module_c.func_c())
    assert module_c.func_c() == 7
    print(module_c.compute_c(5))
    assert module_c.compute_c(5) == 18


def test_module_d() -> None:
    print(module_d.func_d())
    assert module_d.func_d() == 11
    print(module_d.compute_d(5))
    assert module_d.compute_d(5) == 21


def test_nested_bottom() -> None:
    print(bottom.bottom_func())
    assert bottom.bottom_func() == 100
    print(bottom.bottom_compute(10))
    assert bottom.bottom_compute(10) == 110


def test_nested_mid() -> None:
    print(mid_level.mid_func())
    assert mid_level.mid_func() == 110
    print(mid_level.mid_compute(10))
    assert mid_level.mid_compute(10) == 120


def test_nested_top() -> None:
    print(top_level.top_func())
    assert top_level.top_func() == 111
    print(top_level.top_compute(10))
    assert top_level.top_compute(10) == 121


def test_module_e() -> None:
    print(module_e.module_e_func())
    assert module_e.module_e_func() == 326
    print(module_e.module_e_compute(5))
    assert module_e.module_e_compute(5) == 105


def test_module_f() -> None:
    print(module_f.module_f_func())
    assert module_f.module_f_func() == 327
    print(module_f.module_f_compute(10))
    assert module_f.module_f_compute(10) == 230


def test_module_g() -> None:
    result = module_g.module_g_func()
    print(result)
    assert result == 1940
    print(module_g.module_g_use_e())
    assert module_g.module_g_use_e() == 326
    print(module_g.module_g_compute(10))
    assert module_g.module_g_compute(10) == 517


def test_complex_imports_same_dir() -> None:
    print(complex_imports.test_same_dir())
    assert complex_imports.test_same_dir() == 100


def test_complex_imports_parent() -> None:
    print(complex_imports.test_parent_import())
    assert complex_imports.test_parent_import() == 110


def test_complex_imports_grandparent() -> None:
    print(complex_imports.test_grandparent_import())
    assert complex_imports.test_grandparent_import() == 111


def test_complex_imports_great_grandparent() -> None:
    print(complex_imports.test_great_grandparent_import())
    assert complex_imports.test_great_grandparent_import() == 4


def test_complex_imports_specific() -> None:
    print(complex_imports.test_specific_imports())
    assert complex_imports.test_specific_imports() == 328


def test_complex_imports_chain() -> None:
    print(complex_imports.complex_chain())
    assert complex_imports.complex_chain() == 321


def test_complex_imports_compute() -> None:
    print(complex_imports.complex_compute(5))
    assert complex_imports.complex_compute(5) == 118


def test_cross_level() -> None:
    print(cross_level_imports.cross_level_func())
    assert cross_level_imports.cross_level_func() == 322


def test_cross_level_complex() -> None:
    print(cross_level_imports.test_complex_imports_module())
    assert cross_level_imports.test_complex_imports_module() == 100


def test_cross_level_sibling() -> None:
    print(cross_level_imports.test_sibling_interaction())
    assert cross_level_imports.test_sibling_interaction() == 1322


def test_cross_level_renamed() -> None:
    print(cross_level_imports.test_renamed_import())
    assert cross_level_imports.test_renamed_import() == 111


def test_cross_level_chain() -> None:
    print(cross_level_imports.test_deep_chain())
    assert cross_level_imports.test_deep_chain() == 321


def test_cross_level_compute() -> None:
    print(cross_level_imports.compute_cross(5))
    assert cross_level_imports.compute_cross(5) == 24


def test_module_h() -> None:
    print(module_h.module_h_deep_access())
    assert module_h.module_h_deep_access() == 530
    print(module_h.module_h_nested_chain())
    assert module_h.module_h_nested_chain() == 229
    print(module_h.module_h_complex_chain())
    assert module_h.module_h_complex_chain() == 333
    print(module_h.module_h_cross_level())
    assert module_h.module_h_cross_level() == 1652
    print(module_h.module_h_compute(5))
    assert module_h.module_h_compute(5) == 24


def test_deepest_imports_same_dir() -> None:
    print(deepest_imports.test_same_dir())
    assert deepest_imports.test_same_dir() == 1000


def test_deepest_imports_parent() -> None:
    print(deepest_imports.test_parent_import())
    assert deepest_imports.test_parent_import() == 100


def test_deepest_imports_grandparent() -> None:
    print(deepest_imports.test_grandparent_import())
    assert deepest_imports.test_grandparent_import() == 110


def test_deepest_imports_great_grandparent() -> None:
    print(deepest_imports.test_great_grandparent_import())
    assert deepest_imports.test_great_grandparent_import() == 111


def test_deepest_imports_five_level() -> None:
    print(deepest_imports.test_five_level_import())
    assert deepest_imports.test_five_level_import() == 4


def test_deepest_imports_all_levels() -> None:
    print(deepest_imports.test_all_levels())
    assert deepest_imports.test_all_levels() == 1325


def test_deepest_imports_compute() -> None:
    print(deepest_imports.deepest_compute(5))
    assert deepest_imports.deepest_compute(5) == 1012


def test_module_h_deepest() -> None:
    print(module_h.module_h_deepest())
    assert module_h.module_h_deepest() == 1325


def test_import_aliases() -> None:
    print(mod_a.func_a())
    assert mod_a.func_a() == 1


def test_from_import_specific() -> None:
    print(func_a())
    assert func_a() == 1
    print(compute_a(20))
    assert compute_a(20) == 21


def test_from_import_nested() -> None:
    print(bottom_func())
    assert bottom_func() == 100
    print(bottom_compute(5))
    assert bottom_compute(5) == 105


def test_cross_module_class_construct() -> None:
    p: class_provider_a.Vec2 = class_provider_a.Vec2(3, 4)
    print(p.x)
    assert p.x == 3
    print(p.y)
    assert p.y == 4
    print(p.sum())
    assert p.sum() == 7


def test_cross_module_class_method() -> None:
    a: class_provider_a.Vec2 = class_provider_a.Vec2(1, 2)
    b: class_provider_a.Vec2 = class_provider_a.Vec2(3, 4)
    d: int = class_provider_a.dot_vec2(a, b)
    print(d)
    assert d == 11


def test_cross_module_class_field_mutation() -> None:
    p: class_provider_a.Vec2 = class_provider_a.Vec2(5, 6)
    p.x = 50
    print(p.x)
    assert p.x == 50
    print(p.sum())
    assert p.sum() == 56


def test_cross_module_factory_function() -> None:
    p: class_provider_a.Vec2 = class_provider_a.make_vec2(10, 20)
    print(p.x)
    assert p.x == 10
    print(p.y)
    assert p.y == 20


def test_cross_module_same_name_no_collision() -> None:
    a: class_provider_a.Vec2 = class_provider_a.Vec2(1, 2)
    b: class_provider_b.Vec2 = class_provider_b.Vec2(1, 2, 3)
    print(a.sum())
    assert a.sum() == 3
    print(b.sum())
    assert b.sum() == 6


def test_from_import_class() -> None:
    p: ImportedVec2 = ImportedVec2(9, 4)
    print(p.sum())
    assert p.sum() == 13


def test_from_import_nested_class() -> None:
    outer: Outer = Outer(21)
    inner: Outer.Inner = Outer.Inner(8)
    print(outer.get_base())
    assert outer.get_base() == 21
    print(inner.get())
    assert inner.get() == 8


def test_from_import_deep_nested_class() -> None:
    leaf: Deep.Mid.Leaf = Deep.Mid.Leaf(7)
    print(leaf.triple())
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
