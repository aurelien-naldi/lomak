# Logical Model Analysis Kit

This project aims to provide a set of function to define, manipulate, and analyse
qualitative dynamical models based on logical functions.
It is basically a playground for a more efficient implementation of [bioLQM](https://github.com/colomoto/bioLQM)
using rust instead of java.

Beyond being faster, the use of rust will enable the integration of [clean python bindings](https://github.com/aurelien-naldi/lomak-python), to improve the [CoLoMoTo notebook](http://colomoto.org/notebook/) (bioLQM and GINsim rely on py4j for this).


Here are the main underlying objects.


## Boolean functions

Boolean variables are identified by integer UIDs. A variable "group" can be used to associate
them to human-readable names.

Functions can be stored as boolean expressions or prime implicants, with dedicated data structures.
An abstract function is an enum holding one of the supported formats, it provides accessor methods 
to retrieve the function in any format, performing on-demand conversion when needed.

A formula contains an abstract function as main representation, and provides similar accessors to
retrieve any supported format, but it will cache a copy of each requested format to avoid
repeating the same conversion.


## Assignments

TODO: Assignments will be lists of Boolean functions associated to target values. They will be used
to represent multi-valued functions, through the creation of implicit Boolean variables for each
activity threshold.


## Model

A model is a list of assignments (currently of simple Boolean functions), where each variable is
associated to a rule cntrolling it's target value.


## I/O Format

A format handles parsing and exporting models.

