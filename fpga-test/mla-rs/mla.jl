@enum Decision::Int8 begin
    Sell = -1
    Hold = 0
    Buy = 1
end

struct Option{T}
    val::Union{T, Nothing}
end

Option() = Option(nothing)

function tryget(v::Vector{T}, i::Int64)::Union{T, Nothing} where {T}
    1 <= i <= length(v) ? v[i] : nothing
end

const N_INPUTS = 5
const Decisions = NTuple{N_INPUTS, Float64}
const Weights = NTuple{N_INPUTS, Float64}

mutable struct SymWeights
    weights::Vector{Weights}
end

SymWeights(weights::Vector{Weights}=[]) = SymWeights(weights)

function decision(sw::SymWeights, sym_num::Uint16, decisions::Decisions)::Option{Decision}
    const WEIGHT_CUTOFF = 0.33

    weights = tryget(sw.weights, Int64(sym_num)) || return Option()
    res = mapreduce(
        t -> t[1] * decision_to_float(t[2]),
        +,
        zip(weights, decisions),
    ) / Float64(N_INPUTS)
    Option(if res > WEIGHT_CUTOFF
        Buy
    elseif res < -WEIGHT_CUTOFF
        Sell
    else
        Hold
    end)
end

function update!(sw::SymWeights, sym_num::Uint16, correct::NTuple{N_INPUTS, Bool})
    weights = tryget(sw.weights, Int64(sym_num)) || return Option()
    for (i, c) in enumerate(correct)
        # TODO: Adjust
        weights[i] *= c ? 1.1 : 0.95
    end
end
