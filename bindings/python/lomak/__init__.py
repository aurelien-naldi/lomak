from lomak._native import ffi, lib

import sys, ctypes
from ctypes import c_uint32, c_char_p

class Expr:
    def __init__(self, v):
        if isinstance(v,bool):
            self.__ref = lib.new__expr_bool(v)
        elif isinstance(v,int):
            self.__ref = lib.new__expr_var(v)
        else:
            self.__ref = v

    def __str__(self):
        return _str_from_lib(lib.str__expr(self.__ref))
    def __repr__(self):
        return "<Expr: %s>" % self

    def __del__(self):
        lib.drop__expr(self.__ref)

    def simplify(self):
        ret = lib.expr_simplify(self.__ref);
        if ret.ptr:
            return Expr( ret )
        return self

    def nnf(self):
        ret = lib.expr_nnf(self.__ref);
        if ret.ptr:
            return Expr( ret )
        return self

    def dissolve(self, dlink=True):
        ret = lib.expr_dissolve(self.__ref, dlink);
        if ret.ptr:
            return Expr( ret )
        return self

    def __and__(self, other):
        return Expr(lib.expr_and(self.__ref, other.__ref))

    def __or__(self, other):
        return Expr(lib.expr_or(self.__ref, other.__ref))

    def __invert__(self):
        return Expr( lib.expr_not(self.__ref) )


class Model:
    def __init__(self, filename=None):
        if filename:
            self.__ref = lib.load_model(filename.encode('utf-8'))
        else:
            self.__ref = lib.new__model()

    def __del__(self):
        lib.drop__model(self.__ref)

    def rename(self, source,target):
        lib.model_rename(self.__ref, source.encode('utf-8'), target.encode('utf-8'))

    def show(self):
        lib.model_show(self.__ref)

    def primes(self):
        lib.model_primes(self.__ref)

    def fixpoints(self):
        lib.model_fixpoints(self.__ref)

    def expr(self, name):
        return Expr(lib.model_get_expr(self.__ref, name.encode('utf-8')))

    def __getitem__(self, k):
        return Expr(lib.model_get_rule(self.__ref, k.encode('utf-8')))


def _str_from_lib(cdata):
    s = ffi.string(cdata)
    lib.drop__string(cdata)
    return s.decode('utf-8')

