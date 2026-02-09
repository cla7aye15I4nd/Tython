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


def test_module_a() -> None:
    assert module_a.func_a() == 1
    assert module_a.compute_a(10) == 11


def test_module_b() -> None:
    assert module_b.func_b() == 3
    assert module_b.compute_b(5) == 12


def test_module_c() -> None:
    assert module_c.func_c() == 7
    assert module_c.compute_c(5) == 18


def test_module_d() -> None:
    assert module_d.func_d() == 11
    assert module_d.compute_d(5) == 21


def test_nested_bottom() -> None:
    assert bottom.bottom_func() == 100
    assert bottom.bottom_compute(10) == 110


def test_nested_mid() -> None:
    assert mid_level.mid_func() == 110
    assert mid_level.mid_compute(10) == 120


def test_nested_top() -> None:
    assert top_level.top_func() == 111
    assert top_level.top_compute(10) == 121


def test_module_e() -> None:
    assert module_e.module_e_func() == 326
    assert module_e.module_e_compute(5) == 105


def test_module_f() -> None:
    assert module_f.module_f_func() == 327
    assert module_f.module_f_compute(10) == 230


def test_module_g() -> None:
    result = module_g.module_g_func()
    assert result == 1940
    assert module_g.module_g_use_e() == 326
    assert module_g.module_g_compute(10) == 517


def test_complex_imports_same_dir() -> None:
    assert complex_imports.test_same_dir() == 100


def test_complex_imports_parent() -> None:
    assert complex_imports.test_parent_import() == 110


def test_complex_imports_grandparent() -> None:
    assert complex_imports.test_grandparent_import() == 111


def test_complex_imports_great_grandparent() -> None:
    assert complex_imports.test_great_grandparent_import() == 4


def test_complex_imports_specific() -> None:
    assert complex_imports.test_specific_imports() == 328


def test_complex_imports_chain() -> None:
    assert complex_imports.complex_chain() == 321


def test_complex_imports_compute() -> None:
    assert complex_imports.complex_compute(5) == 118


def test_cross_level() -> None:
    assert cross_level_imports.cross_level_func() == 322


def test_cross_level_complex() -> None:
    assert cross_level_imports.test_complex_imports_module() == 100


def test_cross_level_sibling() -> None:
    assert cross_level_imports.test_sibling_interaction() == 1322


def test_cross_level_renamed() -> None:
    assert cross_level_imports.test_renamed_import() == 111


def test_cross_level_chain() -> None:
    assert cross_level_imports.test_deep_chain() == 321


def test_cross_level_compute() -> None:
    assert cross_level_imports.compute_cross(5) == 24


def test_module_h() -> None:
    assert module_h.module_h_deep_access() == 530
    assert module_h.module_h_nested_chain() == 229
    assert module_h.module_h_complex_chain() == 333
    assert module_h.module_h_cross_level() == 1652
    assert module_h.module_h_compute(5) == 24


def test_deepest_imports_same_dir() -> None:
    assert deepest_imports.test_same_dir() == 1000


def test_deepest_imports_parent() -> None:
    assert deepest_imports.test_parent_import() == 100


def test_deepest_imports_grandparent() -> None:
    assert deepest_imports.test_grandparent_import() == 110


def test_deepest_imports_great_grandparent() -> None:
    assert deepest_imports.test_great_grandparent_import() == 111


def test_deepest_imports_five_level() -> None:
    assert deepest_imports.test_five_level_import() == 4


def test_deepest_imports_all_levels() -> None:
    assert deepest_imports.test_all_levels() == 1325


def test_deepest_imports_compute() -> None:
    assert deepest_imports.deepest_compute(5) == 1012


def test_module_h_deepest() -> None:
    assert module_h.module_h_deepest() == 1325


def test_import_aliases() -> None:
    assert mod_a.func_a() == 1


def test_from_import_specific() -> None:
    assert func_a() == 1
    assert compute_a(20) == 21


def test_from_import_nested() -> None:
    assert bottom_func() == 100
    assert bottom_compute(5) == 105


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
