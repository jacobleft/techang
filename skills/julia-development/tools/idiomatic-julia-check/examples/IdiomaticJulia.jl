const DESIGN = quote
    SpecificNoun <: GeneralNoun

    result::ResultNoun = verb(a::NounA, b::NounB)
    mutate!(a::NounA, b::NounB)

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
