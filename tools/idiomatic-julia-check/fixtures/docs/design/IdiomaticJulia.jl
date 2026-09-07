# Noun relationships
const NOUNS = quote
    SpecificNoun <: GeneralNoun
end

#=
Verb declarations may use another quoted block.
=#
const VERBS = quote
    result::ResultNoun = verb(a::NounA, b::NounB)
    mutate!(a::NounA, b::NounB)
end

# Dotted access, including package qualification and object fields
const DOTTED = quote
    Toolkit.AbstractNoun
    Toolkit.ConcreteNoun(a)
    Toolkit.mutate!(a.state, b.cache.value)
    result::Toolkit.ResultNoun = Toolkit.verb(a::Toolkit.NounA, b::Toolkit.NounB)
end

# Extended native Julia notation
const EXTENDED = quote
    verb(a::NounA, b; option::OptionNoun = default)::ResultNoun
    result = verb(a::NounA, b; option = settings.option, mode = :fast)::ResultNoun
    owner.field::FieldNoun

    for (key, value) in pairs(source)
        update!(destination, key, value; mode = settings.mode)
    end

    destination .= source.values
    owner.field += increment
end

# Ordinary Julia algorithm body
const ALGORITHM = quote
    function accumulate!(destination, workspace, item::Item{Family})
        kernel!(destination, workspace.cache, item.data)
    end
end

# Representative composition may use a bare quote.
quote
    result = verb(a, b)
    mutate!(a, b)

    if condition(a)
        result = verb(a, b)
    else
        fallback!(a)
    end

    for item in items(a)
        mutate!(item, b)
    end

    while condition(a)
        mutate!(a, b)
        if finished(a)
            break
        end
    end
end
