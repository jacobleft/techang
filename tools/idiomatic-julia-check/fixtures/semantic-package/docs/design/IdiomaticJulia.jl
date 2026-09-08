quote
    SemanticFixture.transform(
        input::CsvInput,
        options::Options;
        mode::Symbol = :fast,
    )::Output
    result::ExternalResult = Dependency.external(input::ExternalInput)
end
