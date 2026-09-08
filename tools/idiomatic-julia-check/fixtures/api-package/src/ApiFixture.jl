module ApiFixture

abstract type Input end
struct CsvInput <: Input end
struct Options end
struct Output end

function transform(input::Input, options::Options; mode::Symbol = :fast)::Output
    Output()
end

end
