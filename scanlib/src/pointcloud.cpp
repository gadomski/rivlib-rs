#include <riegl/scanlib.hpp>

struct Inclination {
    double time;
    float roll;
    float pitch;
};

class Pointcloud : public scanlib::pointcloud {
public:
    Pointcloud(int32_t sync_to_pps)
    : scanlib::pointcloud(sync_to_pps)
    {}

    void clear() {
        this->m_inclinations.clear();
    }

    virtual void on_hk_incl(const scanlib::hk_incl<iterator_type>& arg) {
        scanlib::pointcloud::on_hk_incl(arg);
        Inclination inclination;
        inclination.time = this->time;
        inclination.roll = float(arg.ROLL) / 1e3;
        inclination.pitch = float(arg.PITCH) / 1e3;
        m_inclinations.push_back(inclination);
    }

    std::vector<Inclination> m_inclinations;
};

class Stream {
public:
    Stream(const char* path, int32_t sync_to_pps)
    : m_connection(scanlib::basic_rconnection::create(path))
    , m_decoder(m_connection)
    , m_pointcloud(sync_to_pps)
    , m_buffer()
    {}

    bool end_of_input() const {
        return this->m_decoder.eoi();
    }

    void read() {
        this->m_decoder.get(this->m_buffer);
        this->m_pointcloud.clear();
        this->m_pointcloud.dispatch(this->m_buffer.begin(), this->m_buffer.end());
    }

    const Inclination* inclinations() const {
        return this->m_pointcloud.m_inclinations.data();
    }

    size_t inclinations_len() const {
        return this->m_pointcloud.m_inclinations.size();
    }

private:
    std::shared_ptr<scanlib::basic_rconnection> m_connection;
    scanlib::decoder_rxpmarker m_decoder;
    Pointcloud m_pointcloud;
    scanlib::buffer m_buffer;
};

extern "C" {
int32_t stream_new(const char* path, int32_t sync_to_pps, Stream** stream) {
    try {
        *stream = new Stream(path, sync_to_pps);
    } catch(std::exception& e) {
        // TODO better error handling
        std::cerr << e.what() << std::endl;
        return 1;
    }
    return 0;
}

int32_t stream_read(Stream* stream, const Inclination** inclinations, size_t* inclinations_len, int32_t* end_of_input) {
    try {
        if (stream->end_of_input()) {
            *end_of_input = 1;
        } else {
            *end_of_input = 0;
            stream->read();
            *inclinations = stream->inclinations();
            *inclinations_len = stream->inclinations_len();
        }
    } catch(std::exception& e) {
        // TODO better error handling
        std::cerr << e.what() << std::endl;
        return 1;
    }
    return 0;
}

int32_t stream_del(Stream* stream) {
    try {
        delete stream;
    } catch(std::exception& e) {
        // TODO better error handling
        std::cerr << e.what() << std::endl;
        return 1;
    }
    return 0;
}
}
