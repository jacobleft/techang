module Dependency

struct ExternalInput end
struct ExternalResult end

function external(input::ExternalInput)::ExternalResult
    ExternalResult()
end

end
